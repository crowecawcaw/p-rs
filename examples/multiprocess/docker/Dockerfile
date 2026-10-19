FROM python:3.13-slim
WORKDIR /app
# Build context is examples/multiprocess. No pip installs or networked build steps.
COPY common/inventory.py common/worker.py ./
ENTRYPOINT ["python", "-u", "/app/worker.py"]
