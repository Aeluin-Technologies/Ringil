#!/bin/bash

echo "Downloading yolo26s-ONNX (FP16)..."
HF_URL="https://huggingface.co/onnx-community/yolo26s-ONNX/resolve/main/onnx/model_fp16.onnx"
curl -L $HF_URL -o yolo26n.onnx

echo "Downloading and extracting buffalo_l..."
GF_URL="https://github.com/deepinsight/insightface/releases/download/v0.7/buffalo_l.zip"
ZIP_NAME="buffalo_l.zip"
TARGET_DIR="buffalo_l"
curl -L $GF_URL -o $ZIP_NAME
mkdir -p $TARGET_DIR
unzip -q $ZIP_NAME -d $TARGET_DIR
rm $ZIP_NAME

echo "Done!"
