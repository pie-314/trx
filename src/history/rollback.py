#!/usr/bin/env python3
"""
Transaction rollback functionality for package management history.

This module provides the ability to revert past package transactions,
supporting both INSTALL and REMOVE operations with version pinning.
"""

import subprocess
import sys
import os
from datetime import datetime
from typing import Optional, Dict, Any
from pathlib import Path

# Import transaction log functionality
from .transaction_log import TransactionLog, TransactionEntry

# Package manager detection
def detect_package_manager() -> str:
    """Detect the system's package manager."""
    if os.path.exists('/usr/bin/apt'):
        return 'apt'
    elif os.path.exists('/usr/bin/pacman'):
        return 'pacman'
    elif os.path.exists('/usr/bin/dnf'):
        return 'dnf'
    elif os.path.exists('/usr/bin/yum'):
        return 'yum'
    else:
        raise RuntimeError("Unsupported package manager")

def get_package_version(package_name: str, package_manager: str) -> Optional[str]:
    """
    Get the installed version of a package.
    
    Args:
        package_name: Name of the package
        package_manager: Package manager to use (apt, pacman, etc.)
    
    Returns:
        Version string or None if not installed
    """
    try:
        if package_manager == 'apt':
            result = subprocess.run(
                ['dpkg-query', '-W', '-f=${Version}', package_name],
                capture_output=True,
                text=True,
                check=True
            )
            return result.stdout.strip()
        elif package_manager == 'pacman':
            result = subprocess.run(
                ['pacman', '-Q', package_name],
                capture_output=True,
                text=True,
                check=True
            )
            # Output format: "package_name version"
            parts = result.stdout.strip().split()
            return parts[1] if len(parts) > 1 else None
        elif package_manager in ('dnf', 'yum'):
            result = subprocess.run(
                ['rpm', '-q', '--queryformat', '%{VERSION}', package_name],
                capture_output=True,
                text=True,
                check=True
            )
            return result.stdout.strip()
    except subprocess.CalledProcessError:
        return None
    return None

def rollback_install(package_name: str, version: Optional[str] = None) -> bool:
    """
    Rollback an INSTALL transaction by removing the package.
    
    Args:
        package_name: Name of the package to remove
        version: Optional version string (not used for removal)
    
    Returns:
        True if successful, False otherwise
    """
    package_manager = detect_package_manager()
    
    try:
        if package_manager == 'apt':
            cmd = ['sudo', 'apt', 'remove', '-y', package_name]
        elif package_manager == 'pacman':
            cmd = ['sudo', 'pacman', '-R', '--noconfirm', package_name]
        elif package_manager in ('dnf', 'yum'):
            cmd = ['sudo', package_manager, 'remove', '-y', package_name]
        else:
            raise RuntimeError(f"Unsupported package manager: {package_manager}")
        
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        print(f"Successfully removed package: {package_name}")
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"Failed to remove package {package_name}: {e.stderr}", file=sys.stderr)
        return False
    except Exception as e:
        print(f"Error during rollback: {e}", file=sys.stderr)
        return False

def rollback_remove(package_name: str, version: Optional[str] = None) -> bool:
    """
    Rollback a REMOVE transaction by reinstalling the package.
    Attempts to install the exact version if provided.
    
    Args:
        package_name: Name of the package to reinstall
        version: Optional version string to pin
    
    Returns:
        True if successful, False otherwise
    """
    package_manager = detect_package_manager()
    
    try:
        if package_manager == 'apt':
            # Try to install with version pinning
            if version:
                # Check if the version is available
                check_cmd = ['apt-cache', 'policy', package_name]
                check_result = subprocess.run(check_cmd, capture_output=True, text=True)
                
                if version in check_result.stdout:
                    cmd = ['sudo', 'apt', 'install', '-y', f'{package_name}={version}']
                else:
                    print(f"Warning: Version {version} not available, installing latest")
                    cmd = ['sudo', 'apt', 'install', '-y', package_name]
            else:
                cmd = ['sudo', 'apt', 'install', '-y', package_name]
                
        elif package_manager == 'pacman':
            # For pacman, try to use cached package or install from repos
            if version:
                # Check if package with version is available
                check_cmd = ['pacman', '-Ss', f'^{package_name}$']
                check_result = subprocess.run(check_cmd, capture_output=True, text=True)
                
                if version in check_result.stdout:
                    cmd = ['sudo', 'pacman', '-S', '--noconfirm', f'{package_name}={version}']
                else:
                    print(f"Warning: Version {version} not available, installing latest")
                    cmd = ['sudo', 'pacman', '-S', '--noconfirm', package_name]
            else:
                cmd = ['sudo', 'pacman', '-S', '--noconfirm', package_name]
                
        elif package_manager in ('dnf', 'yum'):
            if version:
                cmd = ['sudo', package_manager, 'install', '-y', f'{package_name}-{version}']
            else:
                cmd = ['sudo', package_manager, 'install', '-y', package_name]
        else:
            raise RuntimeError(f"Unsupported package manager: {package_manager}")
        
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        print(f"Successfully reinstalled package: {package_name}")
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"Failed to reinstall package {package_name}: {e.stderr}", file=sys.stderr)
        return False
    except Exception as e:
        print(f"Error during rollback: {e}", file=sys.stderr)
        return False

def rollback_transaction(transaction: TransactionEntry) -> bool:
    """
    Rollback a specific transaction.
    
    Args:
        transaction: TransactionEntry object containing transaction details
    
    Returns:
        True if rollback was successful, False otherwise
    """
    print(f"Rolling back transaction #{transaction.id} from {transaction.timestamp}")
    print(f"Action: {transaction.action} - Package: {transaction.package_name}")
    
    if transaction.action == 'INSTALL':
        success = rollback_install(transaction.package_name, transaction.version)
    elif transaction.action == 'REMOVE':
        success = rollback_remove(transaction.package_name, transaction.version)
    else:
        print(f"Unknown transaction action: {transaction.action}", file=sys.stderr)
        return False
    
    if success:
        # Record the rollback as a new transaction
        log = TransactionLog()
        rollback_action = 'ROLLBACK_INSTALL' if transaction.action == 'REMOVE' else 'ROLLBACK_REMOVE'
        log.add_entry(
            action=rollback_action,
            package_name=transaction.package_name,
            version=transaction.version,
            status='SUCCESS'
        )
        print("Rollback completed successfully")
    else:
        print("Rollback failed", file=sys.stderr)
    
    return success

def confirm_rollback(transaction: TransactionEntry) -> bool:
    """
    Ask user for confirmation before performing rollback.
    
    Args:
        transaction: TransactionEntry to rollback
    
    Returns:
        True if user confirms, False otherwise
    """
    print(f"\nWARNING: You are about to rollback transaction #{transaction.id}")
    print(f"This will {'remove' if transaction.action == 'INSTALL' else 'reinstall'} {transaction.package_name}")
    print(f"Timestamp: {transaction.timestamp}")
    
    response = input("Are you sure you want to proceed? (y/N): ").strip().lower()
    return response in ('y', 'yes')

# Main entry point for command-line usage
if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python rollback.py <transaction_id>")
        sys.exit(1)
    
    try:
        transaction_id = int(sys.argv[1])
        log = TransactionLog()
        transaction = log.get_transaction(transaction_id)
        
        if not transaction:
            print(f"Transaction #{transaction_id} not found", file=sys.stderr)
            sys.exit(1)
        
        if confirm_rollback(transaction):
            success = rollback_transaction(transaction)
            sys.exit(0 if success else 1)
        else:
            print("Rollback cancelled")
            sys.exit(0)
            
    except ValueError:
        print("Invalid transaction ID. Please provide a number.", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)