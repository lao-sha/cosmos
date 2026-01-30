"""OpenAI LLM Provider"""

from typing import List, AsyncGenerator
from openai import AsyncOpenAI
from .base import LLMProvider
from ..models import ChatMessage
from ..config import get_settings


class OpenAIProvider(LLMProvider):
    """OpenAI API provider"""
    
    def __init__(self):
        settings = get_settings()
        self.client = AsyncOpenAI(
            api_key=settings.openai_api_key,
            base_url=settings.openai_base_url,
        )
        self.model = settings.openai_model
    
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
        
        response = await self.client.chat.completions.create(
            model=self.model,
            messages=formatted_messages,
            temperature=temperature,
            max_tokens=max_tokens,
        )
        
        return response.choices[0].message.content or ""
    
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
        
        stream = await self.client.chat.completions.create(
            model=self.model,
            messages=formatted_messages,
            temperature=temperature,
            max_tokens=max_tokens,
            stream=True,
        )
        
        async for chunk in stream:
            if chunk.choices[0].delta.content:
                yield chunk.choices[0].delta.content
    
    async def health_check(self) -> bool:
        """Check if the provider is healthy"""
        try:
            await self.client.models.list()
            return True
        except Exception:
            return False
