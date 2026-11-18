from fastapi import FastAPI
from src.services.auth.main import router as auth_router
from src.models import Base
from src.database import engine

app = FastAPI(title="Harbour Backend Services")

# Include routers
app.include_router(auth_router)

app = FastAPI(title="Harbour Backend Services")

@app.get("/")
async def read_root():
    return {"message": "Welcome to Harbour Backend Services API"}
