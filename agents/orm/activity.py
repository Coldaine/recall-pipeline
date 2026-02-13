from datetime import datetime
from typing import Optional

from sqlalchemy import DateTime, String, Text
from sqlalchemy.orm import Mapped, mapped_column

from agents.orm.mixins import DeploymentMixin
from agents.orm.sqlalchemy_base import SqlalchemyBase


class Activity(SqlalchemyBase, DeploymentMixin):
    """Represents a period of coherent activity (e.g., working on database layer)."""

    __tablename__ = "activities"

    id: Mapped[str] = mapped_column(String, primary_key=True)
    start_at: Mapped[Optional[datetime]] = mapped_column(DateTime(timezone=True), nullable=True)
    end_at: Mapped[Optional[datetime]] = mapped_column(DateTime(timezone=True), nullable=True)
    summary: Mapped[Optional[str]] = mapped_column(Text, nullable=True, doc="LLM-generated summary")
    project_id: Mapped[Optional[str]] = mapped_column(
        String, nullable=True, doc="Reference to parent project"
    )
