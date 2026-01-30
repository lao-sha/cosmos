"""Configuration for AI Companion Service"""

from pydantic_settings import BaseSettings
from functools import lru_cache


class Settings(BaseSettings):
    """Application settings"""
    
    # API Settings
    app_name: str = "Meowstar AI Companion"
    debug: bool = False
    api_prefix: str = "/api/v1"
    
    # LLM Settings
    llm_provider: str = "openai"  # openai, ollama, qwen
    openai_api_key: str = ""
    openai_base_url: str = "https://api.openai.com/v1"
    openai_model: str = "gpt-4o-mini"
    
    # Ollama Settings (for local LLM)
    ollama_base_url: str = "http://localhost:11434"
    ollama_model: str = "qwen2.5:7b"
    
    # Embedding Settings
    embedding_model: str = "BAAI/bge-m3"
    embedding_dimension: int = 1024
    
    # Vector Database Settings
    qdrant_host: str = "localhost"
    qdrant_port: int = 6333
    qdrant_collection: str = "meowstar_memories"
    
    # Redis Settings
    redis_url: str = "redis://localhost:6379/0"
    
    # Database Settings
    database_url: str = "postgresql+asyncpg://postgres:postgres@localhost:5432/meowstar"
    
    # Memory Settings
    max_short_term_memory: int = 20
    max_context_tokens: int = 4000
    memory_retrieval_top_k: int = 5
    
    # Rate Limiting
    rate_limit_per_minute: int = 60
    
    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"


@lru_cache
def get_settings() -> Settings:
    """Get cached settings instance"""
    return Settings()
