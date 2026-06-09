#!/usr/bin/env python3
"""
Transaction log module for tracking package management operations.
Supports recording and querying transaction history.
"""

import json
import os
from datetime import datetime
from typing import List, Optional, Dict, Any
from dataclasses import dataclass, asdict
from pathlib import Path

@dataclass
class TransactionEntry:
    """Represents a single transaction entry in the log."""
    id: int
    timestamp: str
    action: str  # INSTALL, REMOVE, ROLLBACK_INSTALL, ROLLBACK_REMOVE
    package_name: str
    version: Optional[str] = None
    status: str = 'SUCCESS'  # SUCCESS, FAILED, PENDING

class TransactionLog:
    """Manages the transaction history log."""
    
    def __init__(self, log_file: Optional[str] = None):
        """
        Initialize the transaction log.
        
        Args:
            log_file: Path to the log file. Defaults to ~/.trx/transactions.json
        """
        if log_file:
            self.log_file = Path(log_file)
        else:
            self.log_file = Path.home() / '.trx' / 'transactions.json'
        
        # Ensure the directory exists
        self.log_file.parent.mkdir(parents=True, exist_ok=True)
        
        # Initialize empty log if file doesn't exist
        if not self.log_file.exists():
            self._save([])
    
    def _load(self) -> List[Dict[str, Any]]:
        """Load transactions from the log file."""
        try:
            with open(self.log_file, 'r') as f:
                return json.load(f)
        except (json.JSONDecodeError, FileNotFoundError):
            return []
    
    def _save(self, transactions: List[Dict[str, Any]]) -> None:
        """Save transactions to the log file."""
        with open(self.log_file, 'w') as f:
            json.dump(transactions, f, indent=2)
    
    def add_entry(self, action: str, package_name: str, 
                  version: Optional[str] = None, 
                  status: str = 'SUCCESS') -> TransactionEntry:
        """
        Add a new transaction entry to the log.
        
        Args:
            action: Type of transaction (INSTALL, REMOVE, etc.)
            package_name: Name of the package
            version: Optional version string
            status: Transaction status
        
        Returns:
            The created TransactionEntry
        """
        transactions = self._load()
        
        # Generate new ID
        new_id = max([t.get('id', 0) for t in transactions], default=0) + 1
        
        entry = TransactionEntry(
            id=new_id,
            timestamp=datetime.now().isoformat(),
            action=action,
            package_name=package_name,
            version=version,
            status=status
        )
        
        transactions.append(asdict(entry))
        self._save(transactions)
        
        return entry
    
    def get_transaction(self, transaction_id: int) -> Optional[TransactionEntry]:
        """
        Get a specific transaction by ID.
        
        Args:
            transaction_id: ID of the transaction to retrieve
        
        Returns:
            TransactionEntry if found, None otherwise
        """
        transactions = self._load()
        for t in transactions:
            if t.get('id') == transaction_id:
                return TransactionEntry(**t)
        return None
    
    def get_all_transactions(self) -> List[TransactionEntry]:
        """
        Get all transactions from the log.
        
        Returns:
            List of TransactionEntry objects
        """
        transactions = self._load()
        return [TransactionEntry(**t) for t in transactions]
    
    def get_transactions_by_package(self, package_name: str) -> List[TransactionEntry]:
        """
        Get all transactions for a specific package.
        
        Args:
            package_name: Name of the package
        
        Returns:
            List of TransactionEntry objects
        """
        transactions = self._load()
        return [
            TransactionEntry(**t) for t in transactions 
            if t.get('package_name') == package_name
        ]
    
    def get_recent_transactions(self, limit: int = 10) -> List[TransactionEntry]:
        """
        Get the most recent transactions.
        
        Args:
            limit: Maximum number of transactions to return
        
        Returns:
            List of TransactionEntry objects
        """
        transactions = self._load()
        # Sort by timestamp descending (most recent first)
        sorted_transactions = sorted(
            transactions, 
            key=lambda x: x.get('timestamp', ''), 
            reverse=True
        )
        return [TransactionEntry(**t) for t in sorted_transactions[:limit]]