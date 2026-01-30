"""Data models for AI Companion Service"""

from datetime import datetime
from enum import Enum
from typing import Optional, List
from pydantic import BaseModel, Field


class Element(str, Enum):
    """Pet element types"""
    NORMAL = "normal"
    FIRE = "fire"
    WATER = "water"
    SHADOW = "shadow"
    LIGHT = "light"


class Rarity(str, Enum):
    """Pet rarity levels"""
    COMMON = "common"
    RARE = "rare"
    EPIC = "epic"
    LEGENDARY = "legendary"
    MYTHIC = "mythic"


class Personality(BaseModel):
    """Pet personality traits"""
    extroversion: int = Field(default=50, ge=0, le=100, description="外向性")
    warmth: int = Field(default=50, ge=0, le=100, description="温暖度")
    humor: int = Field(default=50, ge=0, le=100, description="幽默感")
    curiosity: int = Field(default=50, ge=0, le=100, description="好奇心")
    responsibility: int = Field(default=50, ge=0, le=100, description="责任感")


class PetInfo(BaseModel):
    """Pet information from blockchain"""
    pet_id: int
    owner: str
    name: str
    element: Element
    rarity: Rarity
    level: int
    evolution_stage: int
    personality: Personality
    created_at: datetime


class ChatMessage(BaseModel):
    """Chat message"""
    role: str = Field(..., description="user or assistant")
    content: str
    timestamp: datetime = Field(default_factory=datetime.utcnow)


class ChatRequest(BaseModel):
    """Chat request from user"""
    pet_id: int
    user_id: str
    message: str
    context: Optional[List[ChatMessage]] = None


class ChatResponse(BaseModel):
    """Chat response from AI"""
    pet_id: int
    message: str
    emotion: Optional[str] = None
    action: Optional[str] = None
    timestamp: datetime = Field(default_factory=datetime.utcnow)


class Memory(BaseModel):
    """Memory entry"""
    id: Optional[str] = None
    pet_id: int
    user_id: str
    content: str
    memory_type: str = "conversation"  # conversation, event, preference
    importance: float = 0.5
    embedding: Optional[List[float]] = None
    created_at: datetime = Field(default_factory=datetime.utcnow)
    metadata: Optional[dict] = None


class MemorySearchResult(BaseModel):
    """Memory search result"""
    memory: Memory
    score: float


class UserPreference(BaseModel):
    """User preference"""
    user_id: str
    pet_id: int
    nickname: Optional[str] = None
    interests: List[str] = []
    important_dates: dict = {}
    communication_style: str = "casual"
