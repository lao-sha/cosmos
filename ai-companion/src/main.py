"""Main FastAPI Application"""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from contextlib import asynccontextmanager

from .config import get_settings
from .api import router


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan events"""
    # Startup
    settings = get_settings()
    print(f"Starting {settings.app_name}...")
    print(f"LLM Provider: {settings.llm_provider}")
    print(f"Embedding Model: {settings.embedding_model}")
    
    yield
    
    # Shutdown
    print("Shutting down...")


def create_app() -> FastAPI:
    """Create and configure FastAPI application"""
    settings = get_settings()
    
    app = FastAPI(
        title=settings.app_name,
        description="Meowstar Universe AI Companion Service - 喵星宇宙 AI 陪伴宠物服务",
        version="0.1.0",
        lifespan=lifespan,
    )
    
    # CORS middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )
    
    # Include API routes
    app.include_router(router, prefix=settings.api_prefix)
    
    @app.get("/")
    async def root():
        return {
            "name": settings.app_name,
            "version": "0.1.0",
            "docs": "/docs",
        }
    
    return app


app = create_app()


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "src.main:app",
        host="0.0.0.0",
        port=8000,
        reload=True,
    )
