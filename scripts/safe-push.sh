#!/bin/bash

# Safe Git Push Script for iQube Protocol
# Ensures .env files are in gitignore and pushes to staging branch

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔒 iQube Protocol Safe Git Push Script${NC}"
echo "================================================"

# Function to check if we're in a git repository
check_git_repo() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        echo -e "${RED}❌ Error: Not in a git repository${NC}"
        exit 1
    fi
}

# Function to check .env files in gitignore
check_env_gitignore() {
    local gitignore_file=".gitignore"
    
    if [[ ! -f "$gitignore_file" ]]; then
        echo -e "${RED}❌ Error: .gitignore file not found${NC}"
        exit 1
    fi
    
    # Check for various .env patterns
    local env_patterns=(".env" ".env.local" ".env.development" ".env.production" ".env.staging")
    local missing_patterns=()
    
    for pattern in "${env_patterns[@]}"; do
        if ! grep -q "^${pattern}$" "$gitignore_file" && ! grep -q "^${pattern}$" "$gitignore_file"; then
            missing_patterns+=("$pattern")
        fi
    done
    
    if [[ ${#missing_patterns[@]} -gt 0 ]]; then
        echo -e "${RED}❌ Error: The following .env patterns are missing from .gitignore:${NC}"
        for pattern in "${missing_patterns[@]}"; do
            echo -e "   ${RED}- $pattern${NC}"
        done
        echo -e "${YELLOW}💡 Add these patterns to .gitignore before pushing${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ .env files are properly ignored${NC}"
}

# Function to check for .env files in staging area
check_staged_env_files() {
    local staged_env_files=$(git diff --cached --name-only | grep -E '\.env' || true)
    
    if [[ -n "$staged_env_files" ]]; then
        echo -e "${RED}❌ Error: .env files are staged for commit:${NC}"
        echo "$staged_env_files" | while read -r file; do
            echo -e "   ${RED}- $file${NC}"
        done
        echo -e "${YELLOW}💡 Remove these files from staging: git reset HEAD <file>${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ No .env files in staging area${NC}"
}

# Function to ensure we're pushing to staging branch
ensure_staging_branch() {
    local current_branch=$(git branch --show-current)
    local target_branch="staging"
    
    # Check if staging branch exists
    if ! git show-ref --verify --quiet refs/heads/staging; then
        echo -e "${YELLOW}⚠️  Staging branch doesn't exist. Creating it...${NC}"
        git checkout -b staging
        echo -e "${GREEN}✅ Created staging branch${NC}"
    elif [[ "$current_branch" != "staging" ]]; then
        echo -e "${YELLOW}⚠️  Not on staging branch. Switching to staging...${NC}"
        git checkout staging
        
        # Merge current branch into staging
        echo -e "${BLUE}🔄 Merging $current_branch into staging...${NC}"
        git merge "$current_branch" --no-edit
        echo -e "${GREEN}✅ Merged $current_branch into staging${NC}"
    fi
    
    echo -e "${GREEN}✅ On staging branch${NC}"
}

# Function to push to remote
push_to_remote() {
    local repo_type=""
    local remote_url=$(git remote get-url origin)
    
    # Determine if this is the subapp or monorepo
    if [[ "$remote_url" == *"AigentZBeta"* ]]; then
        repo_type="AigentZ Subapp"
    elif [[ "$remote_url" == *"iQubeBeta-Program"* ]]; then
        repo_type="Monorepo"
    else
        repo_type="Unknown Repository"
    fi
    
    echo -e "${BLUE}📤 Pushing to $repo_type (staging branch)...${NC}"
    
    # Check if remote staging branch exists and pull if needed
    if git ls-remote --heads origin staging | grep -q staging; then
        echo -e "${YELLOW}⚠️  Remote staging branch exists. Pulling latest changes...${NC}"
        git pull origin staging --no-edit || {
            echo -e "${RED}❌ Failed to pull remote staging. Attempting rebase...${NC}"
            git pull --rebase origin staging
        }
    fi
    
    # Push to staging branch
    git push origin staging
    
    echo -e "${GREEN}✅ Successfully pushed to staging branch${NC}"
    echo -e "${YELLOW}💡 Create a PR from staging to main when ready for production${NC}"
}

# Function to show summary
show_summary() {
    local repo_name=$(basename $(git remote get-url origin) .git)
    local commit_hash=$(git rev-parse --short HEAD)
    local commit_message=$(git log -1 --pretty=format:"%s")
    
    echo ""
    echo -e "${GREEN}🎉 Push Summary:${NC}"
    echo "Repository: $repo_name"
    echo "Branch: staging"
    echo "Commit: $commit_hash"
    echo "Message: $commit_message"
    echo ""
    echo -e "${BLUE}🔗 Next Steps:${NC}"
    echo "1. Review changes in staging branch"
    echo "2. Create PR: staging → main"
    echo "3. Test in staging environment"
    echo "4. Merge to main when ready"
}

# Main execution
main() {
    check_git_repo
    check_env_gitignore
    check_staged_env_files
    ensure_staging_branch
    push_to_remote
    show_summary
}

# Run the script
main "$@"
