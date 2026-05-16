#!/bin/bash

echo "Downloading yolo26s-ONNX (FP16)..."
HF_URL="https://github.com/Aeluin-Technologies/models/releases/download/yolo26s/yolo26s.onnx"
curl -L $HF_URL -o yolo26s.onnx

echo "Downloading and extracting buffalo_s..."
GF_URL="https://github.com/deepinsight/insightface/releases/download/v0.7/buffalo_s.zip"
ZIP_NAME="buffalo_s.zip"
TARGET_DIR="buffalo_s"
curl -L $GF_URL -o $ZIP_NAME
mkdir -p $TARGET_DIR
unzip -q $ZIP_NAME -d $TARGET_DIR
rm $ZIP_NAME

echo "Done!"
