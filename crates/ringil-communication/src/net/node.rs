use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;
use zenoh::{Config, Result as ZResult, Session};

use crate::net::crypto::PayloadEncryptor;
use crate::protocol::pb::RingilFrame;

/// Defines the routing topology for a message.
#[derive(Debug, Clone, Copy)]
pub enum SwarmTarget {
    /// Broadcast to the entire swarm.
    Global,
    /// Multicast to a specific sub-group/mission cluster.
    Cluster(u32),
    /// Unicast to a specific drone.
    Node(u32),
}

impl SwarmTarget {
    /// Generates the Zenoh key expression for publishing.
    fn to_key_expr(&self, sender_id: u32) -> String {
        match self {
            SwarmTarget::Global => format!("ringil/swarm/global/{sender_id}"),
            SwarmTarget::Cluster(cid) => {
                format!("ringil/swarm/cluster/{cid}/{sender_id}")
            },
            SwarmTarget::Node(target_id) => {
                format!("ringil/swarm/node/{target_id}/{sender_id}")
            },
        }
    }
}

pub struct SwarmNode {
    session: Session,
    encryptor: Arc<PayloadEncryptor>,
    pub drone_id: u32,
}

impl SwarmNode {
    /// Initialize Zenoh node.
    ///
    /// `lora_iface` should be the network interface name (e.g., "tun0").
    pub async fn new(
        drone_id: u32,
        tpm_key: &[u8; 32],
        lora_iface: &str,
    ) -> ZResult<Self> {
        let mut config = Config::default();
        config.insert_json5("listen/endpoints", r#"["udp/0.0.0.0:7447"]"#)?;
        config.insert_json5("scouting/multicast/enabled", "true")?;
        config.insert_json5(
            "scouting/multicast/interfaces",
            &format!(r#"["{lora_iface}"]"#),
        )?;

        let session = zenoh::open(config).await?;
        let encryptor = Arc::new(PayloadEncryptor::new(tpm_key));

        Ok(Self {
            session,
            encryptor,
            drone_id,
        })
    }

    /// Encrypts and publishes a frame to a specific swarm topology.
    pub async fn publish(
        &self,
        target: SwarmTarget,
        frame: RingilFrame,
    ) -> Result<()> {
        let payload = self.encryptor.encrypt(&frame)?;
        let key_expr = target.to_key_expr(self.drone_id);

        self.session
            .put(&key_expr, payload)
            .await
            .map_err(|e| anyhow::anyhow!("Zenoh publish failed: {}", e))?;

        Ok(())
    }

    /// Subscribes to messages relevant to this node.
    pub async fn subscribe_system(
        &self,
        current_cluster: Option<u32>,
    ) -> Result<mpsc::Receiver<RingilFrame>> {
        // Can handle bursts from 3 combined topics.
        let (tx, rx) = mpsc::channel(100);

        let mut topics = vec![
            "ringil/swarm/global/*".to_string(),
            format!("ringil/swarm/node/{}/*", self.drone_id),
        ];

        if let Some(cid) = current_cluster {
            topics.push(format!("ringil/swarm/cluster/{cid}/*"));
        }

        for topic in topics {
            let tx_clone = tx.clone();
            let encryptor = Arc::clone(&self.encryptor);
            let topic_name = topic.clone();

            let subscriber =
                self.session.declare_subscriber(&topic).await.map_err(
                    |e| {
                        anyhow::anyhow!(
                            "Failed to declare subscriber on {topic}: {e}"
                        )
                    },
                )?;

            // Spawn an independent task for each Zenoh subscription.
            tokio::spawn(async move {
                while let Ok(sample) = subscriber.recv_async().await {
                    let payload_bytes = sample.payload().to_bytes();

                    match encryptor.decrypt(&payload_bytes) {
                        Ok(frame) => {
                            // If receiver is dropped this fails and breaks the loop.
                            if tx_clone.send(frame).await.is_err() {
                                tracing::info!(
                                    ?topic_name, "subscribed to a topic"
                                );
                                break;
                            }
                        },
                        Err(_) => {
                            tracing::warn!(
                                ?topic_name, "dropped unauthorized packet"
                            );
                        },
                    }
                }
            });
        }

        Ok(rx)
    }
}
