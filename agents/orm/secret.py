from datetime import datetime
from typing import Optional

from sqlalchemy import LargeBinary, String
from sqlalchemy.orm import Mapped, mapped_column

from agents.orm.mixins import DeploymentMixin
from agents.orm.sqlalchemy_base import SqlalchemyBase


class Secret(SqlalchemyBase, DeploymentMixin):
    """
    Stores detected secrets.

    WARNING: This table currently stores secrets in plaintext.
    FIDO2 hardware key encryption is the goal, but is not yet implemented.
    """

    __tablename__ = "secrets"

    id: Mapped[str] = mapped_column(String, primary_key=True)
    frame_id: Mapped[Optional[str]] = mapped_column(
        String, nullable=True, doc="Reference to frame where secret was detected"
    )
    secret_type: Mapped[str] = mapped_column(
        String, nullable=False, doc="Type of secret: api_key, password, token, etc."
    )
    raw_value: Mapped[bytes] = mapped_column(
        LargeBinary, nullable=False, doc="Unencrypted secret value"
    )
    key_id: Mapped[Optional[str]] = mapped_column(
        String,
        nullable=True,
        doc="Future: Which hardware key will be used for encryption",
    )
    detected_at: Mapped[datetime] = mapped_column(
        nullable=False, doc="When secret was detected"
    )
