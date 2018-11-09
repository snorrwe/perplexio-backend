#usr/bin/bash
docker build -t frenetiq/perplexio-backend .
echo "$DOCKER_PASSWORD" | docker login -u "$DOCKER_USERNAME" --password-stdin
docker push frenetiq/perplexio-backend
