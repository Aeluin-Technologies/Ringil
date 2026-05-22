{
  lib,
  rustPlatform,
  fetchCrate,
  pkg-config,
  openssl,
  perl,
  stdenv,
}:
rustPlatform.buildRustPackage rec {
  pname = "dora-cli";
  version = "0.5.0";

  src = fetchCrate {
    inherit pname version;
    hash = "sha256-Jtz0JFNlZbqb6Nz9vrOg+TwFpXjb233ACLzmqDcD7MM=";
  };

  cargoHash = "sha256-sQERDG0msA5iFxCyS71/3zL0Q0SwU1hKK3QoFW/ThGc=";

  nativeBuildInputs = [pkg-config perl];
  buildInputs = [openssl];

  doCheck = false;

  meta = with lib; {
    description = "`dora` goal is to be a low latency, composable, and distributed data flow.";
    homepage = "https://dora-rs.ai";
    license = licenses.asl20;
    maintainers = [];
  };
}
