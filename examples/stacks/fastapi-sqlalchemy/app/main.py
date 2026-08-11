from fastapi import FastAPI
from sqlalchemy.orm import DeclarativeBase

app = FastAPI()


class Base(DeclarativeBase):
    pass


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
