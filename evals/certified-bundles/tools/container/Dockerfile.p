ARG BASE_IMAGE=snappea-eval:py312-pytest842-base
FROM ${BASE_IMAGE}

COPY pysnappea-0.1.0-py3-none-any.whl /tmp/pysnappea-0.1.0-py3-none-any.whl
RUN python -m pip install --no-index /tmp/pysnappea-0.1.0-py3-none-any.whl \
    && rm -f /tmp/pysnappea-0.1.0-py3-none-any.whl

WORKDIR /workspace
