"""API Routes for AI Companion Service"""

from fastapi import APIRouter, HTTPException, Depends
from fastapi.responses import StreamingResponse
from typing import List
import json

from ..models import (
    ChatRequest, ChatResponse, Memory, PetInfo,
    Personality, Element, Rarity
)
from ..chat import ChatService
from ..memory import MemoryManager
from ..llm import LLMRouter
from ..config import get_settings, Settings

router = APIRouter()

# Service instances
chat_service = ChatService()
memory_manager = MemoryManager()
llm_router = LLMRouter()


def get_chat_service() -> ChatService:
    return chat_service


def get_memory_manager() -> MemoryManager:
    return memory_manager


@router.get("/health")
async def health_check():
    """Health check endpoint"""
    llm_health = await llm_router.health_check()
    return {
        "status": "healthy",
        "llm_providers": llm_health,
    }


@router.post("/chat", response_model=ChatResponse)
async def chat(
    request: ChatRequest,
    service: ChatService = Depends(get_chat_service),
):
    """Send a chat message and get response"""
    try:
        response = await service.chat(request)
        return response
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/chat/stream")
async def chat_stream(
    request: ChatRequest,
    service: ChatService = Depends(get_chat_service),
):
    """Stream chat response"""
    async def generate():
        try:
            async for chunk in service.chat_stream(request):
                yield f"data: {json.dumps({'content': chunk})}\n\n"
            yield "data: [DONE]\n\n"
        except Exception as e:
            yield f"data: {json.dumps({'error': str(e)})}\n\n"
    
    return StreamingResponse(
        generate(),
        media_type="text/event-stream",
    )


@router.get("/chat/greeting/{pet_id}")
async def get_greeting(
    pet_id: int,
    user_id: str,
    service: ChatService = Depends(get_chat_service),
):
    """Get a personalized greeting"""
    greeting = await service.get_greeting(pet_id, user_id)
    return {"greeting": greeting}


@router.post("/chat/clear/{pet_id}")
async def clear_conversation(
    pet_id: int,
    user_id: str,
    service: ChatService = Depends(get_chat_service),
):
    """Clear conversation context"""
    await service.clear_conversation(pet_id, user_id)
    return {"status": "cleared"}


@router.get("/memory/{pet_id}", response_model=List[Memory])
async def get_memories(
    pet_id: int,
    user_id: str,
    limit: int = 100,
    manager: MemoryManager = Depends(get_memory_manager),
):
    """Get all memories for a pet"""
    memories = await manager.get_all_memories(pet_id, user_id, limit)
    return memories


@router.post("/memory/{pet_id}")
async def add_memory(
    pet_id: int,
    user_id: str,
    content: str,
    memory_type: str = "manual",
    importance: float = 0.5,
    manager: MemoryManager = Depends(get_memory_manager),
):
    """Manually add a memory"""
    memory_id = await manager.save_to_long_term(
        pet_id=pet_id,
        user_id=user_id,
        content=content,
        memory_type=memory_type,
        importance=importance,
    )
    return {"memory_id": memory_id}


@router.delete("/memory/{memory_id}")
async def delete_memory(
    memory_id: str,
    manager: MemoryManager = Depends(get_memory_manager),
):
    """Delete a specific memory"""
    success = await manager.delete_memory(memory_id)
    return {"deleted": success}


@router.post("/memory/search/{pet_id}")
async def search_memories(
    pet_id: int,
    user_id: str,
    query: str,
    top_k: int = 5,
    manager: MemoryManager = Depends(get_memory_manager),
):
    """Search memories by semantic similarity"""
    results = await manager.retrieve_relevant_memories(
        pet_id=pet_id,
        user_id=user_id,
        query=query,
        top_k=top_k,
    )
    return {
        "results": [
            {"memory": r.memory, "score": r.score}
            for r in results
        ]
    }


@router.post("/pet/sync")
async def sync_pet_info(
    pet_info: PetInfo,
    service: ChatService = Depends(get_chat_service),
):
    """Sync pet info from blockchain"""
    await service.update_pet_info(pet_info)
    return {"status": "synced"}
