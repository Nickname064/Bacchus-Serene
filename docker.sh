#!/bin/bash

docker stop bacchus-serene
docker remove bacchus-serene
docker build . --tag nikolai/bacchus-serene:latest
docker run --name bacchus-serene -p 8080:8080 --restart unless-stopped -v ./runtime_data:/app/data -d nikolai/bacchus-serene:latest
