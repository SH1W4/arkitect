from fastapi import FastAPI
from api import ark_routes

app = FastAPI()

app.include_router(ark_routes.router, prefix="/api", tags=["ARK_Σ_Module"])
