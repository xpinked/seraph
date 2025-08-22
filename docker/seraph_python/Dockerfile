FROM python:3.12-slim

WORKDIR /app

COPY ./docker/seraph_python/structure/ /app/

RUN apt update
RUN pip install uv

CMD ["python", "main.py"]
