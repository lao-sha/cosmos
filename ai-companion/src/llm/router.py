"""LLM Router - Routes requests to appropriate LLM provider"""

from typing import List, AsyncGenerator, Optional
from .base import LLMProvider
from .openai_provider import OpenAIProvider
from .ollama_provider import OllamaProvider
from ..models import ChatMessage
from ..config import get_settings


class LLMRouter:
    """Routes LLM requests to appropriate provider with fallback"""
    
    def __init__(self):
        settings = get_settings()
        self.primary_provider: str = settings.llm_provider
        self.providers: dict[str, LLMProvider] = {}
        
        # Initialize providers
        if settings.openai_api_key:
            self.providers["openai"] = OpenAIProvider()
        self.providers["ollama"] = OllamaProvider()
    
    def get_provider(self, provider_name: Optional[str] = None) -> LLMProvider:
        """Get a specific provider or the primary one"""
        name = provider_name or self.primary_provider
        if name in self.providers:
            return self.providers[name]
        # Fallback to first available
        if self.providers:
            return next(iter(self.providers.values()))
        raise RuntimeError("No LLM providers available")
    
    async def chat(
        self,
        messages: List[ChatMessage],
        system_prompt: str,
        temperature: float = 0.7,
        max_tokens: int = 1000,
        provider: Optional[str] = None,
    ) -> str:
        """Generate a chat response with fallback"""
        # Try primary provider
        try:
            llm = self.get_provider(provider)
            return await llm.chat(messages, system_prompt, temperature, max_tokens)
        except Exception as e:
            # Fallback to other providers
            for name, llm in self.providers.items():
                if name != (provider or self.primary_provider):
                    try:
                        return await llm.chat(messages, system_prompt, temperature, max_tokens)
                    except Exception:
                        continue
            raise e
    
    async def chat_stream(
        self,
        messages: List[ChatMessage],
        system_prompt: str,
        temperature: float = 0.7,
        max_tokens: int = 1000,
        provider: Optional[str] = None,
    ) -> AsyncGenerator[str, None]:
        """Generate a streaming chat response"""
        llm = self.get_provider(provider)
        async for chunk in llm.chat_stream(messages, system_prompt, temperature, max_tokens):
            yield chunk
    
    async def health_check(self) -> dict[str, bool]:
        """Check health of all providers"""
        results = {}
        for name, provider in self.providers.items():
            results[name] = await provider.health_check()
        return results
