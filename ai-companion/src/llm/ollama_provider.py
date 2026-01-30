"""Ollama LLM Provider for local models"""

from typing import List, AsyncGenerator
import httpx
from .base import LLMProvider
from ..models import ChatMessage
from ..config import get_settings


class OllamaProvider(LLMProvider):
    """Ollama local LLM provider"""
    
    def __init__(self):
        settings = get_settings()
        self.base_url = settings.ollama_base_url
        self.model = settings.ollama_model
    
    async def chat(
        self,
        messages: List[ChatMessage],
        system_prompt: str,
        temperature: float = 0.7,
        max_tokens: int = 1000,
    ) -> str:
        """Generate a chat response"""
        formatted_messages = [{"role": "system", "content": system_prompt}]
        for msg in messages:
            formatted_messages.append({
                "role": msg.role,
                "content": msg.content,
            })
        
        async with httpx.AsyncClient(timeout=60.0) as client:
            response = await client.post(
                f"{self.base_url}/api/chat",
                json={
                    "model": self.model,
                    "messages": formatted_messages,
                    "stream": False,
                    "options": {
                        "temperature": temperature,
                        "num_predict": max_tokens,
                    },
                },
            )
            response.raise_for_status()
            data = response.json()
            return data.get("message", {}).get("content", "")
    
    async def chat_stream(
        self,
        messages: List[ChatMessage],
        system_prompt: str,
        temperature: float = 0.7,
        max_tokens: int = 1000,
    ) -> AsyncGenerator[str, None]:
        """Generate a streaming chat response"""
        formatted_messages = [{"role": "system", "content": system_prompt}]
        for msg in messages:
            formatted_messages.append({
                "role": msg.role,
                "content": msg.content,
            })
        
        async with httpx.AsyncClient(timeout=60.0) as client:
            async with client.stream(
                "POST",
                f"{self.base_url}/api/chat",
                json={
                    "model": self.model,
                    "messages": formatted_messages,
                    "stream": True,
                    "options": {
                        "temperature": temperature,
                        "num_predict": max_tokens,
                    },
                },
            ) as response:
                async for line in response.aiter_lines():
                    if line:
                        import json
                        data = json.loads(line)
                        if content := data.get("message", {}).get("content"):
                            yield content
    
    async def health_check(self) -> bool:
        """Check if the provider is healthy"""
        try:
            async with httpx.AsyncClient(timeout=5.0) as client:
                response = await client.get(f"{self.base_url}/api/tags")
                return response.status_code == 200
        except Exception:
            return False
