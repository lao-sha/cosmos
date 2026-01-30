"""Chat Service - Core chat functionality"""

from typing import List, Optional, AsyncGenerator
from datetime import datetime
from ..models import (
    PetInfo, ChatMessage, ChatRequest, ChatResponse,
    Personality, Element, Rarity
)
from ..llm import LLMRouter
from ..memory import MemoryManager
from ..prompt import PersonalityPromptBuilder


class ChatService:
    """Main chat service for AI companion"""
    
    def __init__(self):
        self.llm_router = LLMRouter()
        self.memory_manager = MemoryManager()
        self._pet_cache: dict[int, PetInfo] = {}
    
    async def chat(self, request: ChatRequest) -> ChatResponse:
        """Process a chat request and generate response"""
        # Get pet info (from cache or blockchain)
        pet_info = await self._get_pet_info(request.pet_id, request.user_id)
        
        # Build prompt
        prompt_builder = PersonalityPromptBuilder(pet_info)
        
        # Get memory context
        context = await self.memory_manager.get_context_for_chat(
            pet_id=request.pet_id,
            user_id=request.user_id,
            current_message=request.message,
        )
        
        # Build system prompt with memories
        system_prompt = prompt_builder.build_system_prompt(
            memories=context["long_term"]
        )
        
        # Prepare messages
        messages = list(context["short_term"])
        messages.append(ChatMessage(
            role="user",
            content=request.message,
            timestamp=datetime.utcnow(),
        ))
        
        # Generate response
        response_text = await self.llm_router.chat(
            messages=messages,
            system_prompt=system_prompt,
            temperature=0.8,
        )
        
        # Create response message
        assistant_message = ChatMessage(
            role="assistant",
            content=response_text,
            timestamp=datetime.utcnow(),
        )
        
        # Update short-term memory
        self.memory_manager.add_to_short_term(
            request.pet_id, request.user_id,
            ChatMessage(role="user", content=request.message)
        )
        self.memory_manager.add_to_short_term(
            request.pet_id, request.user_id,
            assistant_message
        )
        
        # Save important conversations to long-term memory
        await self._maybe_save_to_long_term(
            request.pet_id, request.user_id,
            request.message, response_text
        )
        
        return ChatResponse(
            pet_id=request.pet_id,
            message=response_text,
            emotion=self._detect_emotion(response_text),
            timestamp=datetime.utcnow(),
        )
    
    async def chat_stream(
        self,
        request: ChatRequest,
    ) -> AsyncGenerator[str, None]:
        """Stream chat response"""
        pet_info = await self._get_pet_info(request.pet_id, request.user_id)
        prompt_builder = PersonalityPromptBuilder(pet_info)
        
        context = await self.memory_manager.get_context_for_chat(
            pet_id=request.pet_id,
            user_id=request.user_id,
            current_message=request.message,
        )
        
        system_prompt = prompt_builder.build_system_prompt(
            memories=context["long_term"]
        )
        
        messages = list(context["short_term"])
        messages.append(ChatMessage(
            role="user",
            content=request.message,
            timestamp=datetime.utcnow(),
        ))
        
        full_response = ""
        async for chunk in self.llm_router.chat_stream(
            messages=messages,
            system_prompt=system_prompt,
            temperature=0.8,
        ):
            full_response += chunk
            yield chunk
        
        # Update memory after streaming completes
        self.memory_manager.add_to_short_term(
            request.pet_id, request.user_id,
            ChatMessage(role="user", content=request.message)
        )
        self.memory_manager.add_to_short_term(
            request.pet_id, request.user_id,
            ChatMessage(role="assistant", content=full_response)
        )
    
    async def _get_pet_info(self, pet_id: int, user_id: str) -> PetInfo:
        """Get pet info from cache or create default"""
        if pet_id in self._pet_cache:
            return self._pet_cache[pet_id]
        
        # TODO: Fetch from blockchain via RPC
        # For now, create a default pet
        pet_info = PetInfo(
            pet_id=pet_id,
            owner=user_id,
            name=f"喵星{pet_id}号",
            element=Element.NORMAL,
            rarity=Rarity.COMMON,
            level=1,
            evolution_stage=0,
            personality=Personality(),
            created_at=datetime.utcnow(),
        )
        
        self._pet_cache[pet_id] = pet_info
        return pet_info
    
    async def update_pet_info(self, pet_info: PetInfo):
        """Update cached pet info"""
        self._pet_cache[pet_info.pet_id] = pet_info
    
    async def _maybe_save_to_long_term(
        self,
        pet_id: int,
        user_id: str,
        user_message: str,
        assistant_message: str,
    ):
        """Determine if conversation should be saved to long-term memory"""
        # Simple heuristic: save if message is long or contains keywords
        important_keywords = [
            "记住", "别忘了", "重要", "生日", "喜欢", "讨厌",
            "remember", "important", "birthday", "love", "hate"
        ]
        
        combined = user_message + assistant_message
        is_important = (
            len(user_message) > 50 or
            any(kw in combined.lower() for kw in important_keywords)
        )
        
        if is_important:
            await self.memory_manager.save_to_long_term(
                pet_id=pet_id,
                user_id=user_id,
                content=f"用户: {user_message}\n宠物: {assistant_message[:100]}...",
                memory_type="conversation",
                importance=0.7,
            )
    
    def _detect_emotion(self, text: str) -> Optional[str]:
        """Simple emotion detection from response"""
        emotion_keywords = {
            "happy": ["开心", "高兴", "太好了", "哈哈", "😊", "🎉"],
            "sad": ["难过", "伤心", "可惜", "😢", "💔"],
            "excited": ["太棒了", "激动", "兴奋", "！！", "🎊"],
            "caring": ["关心", "担心", "照顾", "休息", "💕"],
            "curious": ["好奇", "为什么", "怎么", "🤔"],
        }
        
        for emotion, keywords in emotion_keywords.items():
            if any(kw in text for kw in keywords):
                return emotion
        
        return "neutral"
    
    async def get_greeting(self, pet_id: int, user_id: str) -> str:
        """Get a personalized greeting"""
        pet_info = await self._get_pet_info(pet_id, user_id)
        prompt_builder = PersonalityPromptBuilder(pet_info)
        return prompt_builder.get_greeting()
    
    async def clear_conversation(self, pet_id: int, user_id: str):
        """Clear current conversation context"""
        # Summarize and save before clearing
        short_term = self.memory_manager.get_short_term(pet_id, user_id)
        if short_term:
            await self.memory_manager.summarize_and_store(
                pet_id, user_id, short_term
            )
        
        self.memory_manager.clear_short_term(pet_id, user_id)
