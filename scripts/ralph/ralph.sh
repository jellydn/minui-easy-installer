#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [max_iterations] [cli_tool] [model] [share]
# Supported tools: amp, opencode, mino, mimo, kilo, pi, agy, cmd, codex, copilot, claude
# Requires: bash 4+ (macOS: brew install bash)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ─────────────────────────────────────────────────────────────
# Tool configuration - single source of truth
# Returns: cmd default_model model_flag extra_args permission_cmd prompt_fallback supports_share
# ─────────────────────────────────────────────────────────────
get_tool_config() {
	local tool="$1"
	case "$tool" in
		claude)
			echo "claude||--dangerously-skip-permissions --print||||false"
			;;
		opencode)
			echo "opencode run|opencode/big-pickle|-m|--agent build|export OPENCODE_PERMISSION='{\"*\": \"allow\"}'; export OPENCODE_DISABLE_AUTOCOMPACT=true||true"
			;;
		mino)
			echo "mino run|opencode/big-pickle|-m|--agent build|export MINO_PERMISSION='{\"*\": \"allow\"}'; export MINO_DISABLE_AUTOCOMPACT=true||true"
			;;
		mimo)
			echo "mimo run|mimo/mimo-auto|-m|--agent build|export MINO_PERMISSION='{\"*\": \"allow\"}'; export MINO_DISABLE_AUTOCOMPACT=true|prompt-mino.md|true"
			;;
		kilo)
			echo "kilo run|kilo/kilo-auto|-m|--agent build|export MINO_PERMISSION='{\"*\": \"allow\"}'; export MINO_DISABLE_AUTOCOMPACT=true|prompt-mino.md|true"
			;;
		pi)
			echo "pi||--model|-p|export PI_PERMISSION='{\"*\": \"allow\"}'||false"
			;;
		agy)
			echo "agy||--model|--print --dangerously-skip-permissions|||false"
			;;
		cmd)
			echo "cmd||--model|--print --yolo --skip-onboarding|||false"
			;;
		codex)
			echo "codex exec||-m|--dangerously-bypass-approvals-and-sandbox -|||false"
			;;
		copilot)
			echo "copilot||--model|--yolo -s|||false"
			;;
		amp)
			echo "amp --dangerously-allow-all||--mode||||false"
			;;
		*)
			echo ""
			;;
	esac
}

# ─────────────────────────────────────────────────────────────
# Helper functions
# ─────────────────────────────────────────────────────────────

show_help() {
	cat << 'EOF'
Ralph Wiggum - Long-running AI agent loop

Usage:
  ./ralph.sh [max_iterations] [cli_tool] [model] [share]

Arguments:
  max_iterations    Number of iterations to run (default: 10)
  cli_tool          CLI tool to use (default: amp)
  model             Model ID or mode (tool-specific)
  share             Share session: true/false (default: false)

Supported tools:
  amp         Amp CLI (default)
  opencode    OpenCode CLI
  mino        Mino CLI
  mimo        MiMo CLI
  kilo        Kilo CLI
  pi          Pi CLI
  agy         Agy CLI
  cmd         Command Code CLI
  codex       Codex CLI
  copilot     GitHub Copilot CLI
  claude      Claude Code CLI

Options:
  -h, --help       Show this help message and exit

Examples:
  ./ralph.sh                                    # amp, 10 iterations
  ./ralph.sh 5 opencode                         # OpenCode
  ./ralph.sh 10 mimo mimo/mimo-auto true        # MiMo with share
  ./ralph.sh 10 kilo kilo/kilo-auto true        # Kilo with share
  ./ralph.sh 10 pi google/gemini-2.0-flash      # Pi
  ./ralph.sh 10 agy claude-sonnet               # Agy
  ./ralph.sh 10 cmd claude-sonnet               # Command Code
  ./ralph.sh 10 codex o3                        # Codex
  ./ralph.sh 10 copilot gpt-5.2                 # Copilot
  ./ralph.sh 10 claude                          # Claude Code

Files:
  prompt-*.md       System prompts per CLI tool
  prd.json          Product requirements in Ralph format
  progress.txt      Progress log of completed stories

Completion Signal:
  Ralph stops when the agent outputs: <promise>COMPLETE</promise>
EOF
}

# Resolve prompt file with fallback
resolve_prompt_file() {
	local tool="$1"
	local fallback="$2"
	local prompt_file="$SCRIPT_DIR/prompt-$tool.md"

	if [ ! -f "$prompt_file" ] && [ -n "$fallback" ]; then
		prompt_file="$SCRIPT_DIR/$fallback"
	fi

	echo "$prompt_file"
}

# Execute tool - single generic function
execute_tool() {
	local tool="$1"
	local model="$2"
	local share="$3"
	local prompt_file="$4"
	local config="$5"

	# Parse config (pipe-delimited)
	IFS='|' read -r cmd default_model model_flag extra_args permission_cmd _ supports_share <<< "$config"

	# Use default model if none specified
	[ -z "$model" ] && model="$default_model"

	# Set permissions
	[ -n "$permission_cmd" ] && eval "$permission_cmd"

	# Build model flag
	local model_arg=""
	if [ -n "$model" ] && [ -n "$model_flag" ]; then
		model_arg="$model_flag $model"
	fi

	# Build share flag
	local share_arg=""
	if [ "$share" = "true" ] && [ "$supports_share" = "true" ]; then
		share_arg="--share"
	fi

	# Special handling for copilot (reads prompt via -p flag, not stdin)
	if [ "$tool" = "copilot" ]; then
		local prompt_content
		prompt_content=$(cat "$prompt_file")
		local full_cmd="$cmd $model_arg $extra_args $share_arg -p \"$prompt_content\""
		eval "$full_cmd" 2>&1 | tee /dev/stderr
	else
		# Standard: pipe prompt via stdin
		local full_cmd="$cmd $model_arg $extra_args $share_arg"
		cat "$prompt_file" | eval "$full_cmd" 2>&1 | tee /dev/stderr
	fi
}

# Archive previous run if branch changed
archive_previous_run() {
	local prd_file="$1"
	local progress_file="$2"
	local archive_dir="$3"
	local last_branch_file="$4"

	[ ! -f "$prd_file" ] || [ ! -f "$last_branch_file" ] && return 0

	local current_branch last_branch
	current_branch=$(jq -r '.branchName // empty' "$prd_file" 2>/dev/null || echo "")
	last_branch=$(cat "$last_branch_file" 2>/dev/null || echo "")

	if [ -n "$current_branch" ] && [ -n "$last_branch" ] && [ "$current_branch" != "$last_branch" ]; then
		local date folder_name archive_folder
		date=$(date +%Y-%m-%d)
		folder_name=$(echo "$last_branch" | sed 's|^ralph/||')
		archive_folder="$archive_dir/$date-$folder_name"

		echo "Archiving previous run: $last_branch"
		mkdir -p "$archive_folder"
		cp "$prd_file" "$archive_folder/"
		[ -f "$progress_file" ] && cp "$progress_file" "$archive_folder/"
		echo "   Archived to: $archive_folder"

		# Reset progress file
		echo "# Ralph Progress Log" >"$progress_file"
		echo "Started: $(date)" >>"$progress_file"
		echo "---" >>"$progress_file"
	fi
}

# Track current branch from PRD
track_branch() {
	local prd_file="$1"
	local last_branch_file="$2"

	[ ! -f "$prd_file" ] && return 0

	local current_branch
	current_branch=$(jq -r '.branchName // empty' "$prd_file" 2>/dev/null || echo "")
	[ -n "$current_branch" ] && echo "$current_branch" >"$last_branch_file"
}

# Initialize progress file if missing
init_progress_file() {
	local progress_file="$1"

	if [ ! -f "$progress_file" ]; then
		echo "# Ralph Progress Log" >"$progress_file"
		echo "Started: $(date)" >>"$progress_file"
		echo "---" >>"$progress_file"
	fi
}

# ─────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────

# Parse --help before positional args
for arg in "$@"; do
	if [ "$arg" = "--help" ] || [ "$arg" = "-h" ]; then
		show_help
		exit 0
	fi
done

MAX_ITERATIONS=${1:-10}
CLI_TOOL=${2:-amp}
MODEL=${3:-}
SHARE=${4:-false}

# Validate tool
CONFIG=$(get_tool_config "$CLI_TOOL")
if [ -z "$CONFIG" ]; then
	echo "Error: Unknown CLI tool '$CLI_TOOL'"
	echo "Run with --help to see supported tools."
	exit 1
fi

PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"

# Parse config for prompt fallback
IFS='|' read -r _ _ _ _ _ PROMPT_FALLBACK _ <<< "$CONFIG"
PROMPT_FILE=$(resolve_prompt_file "$CLI_TOOL" "$PROMPT_FALLBACK")

# Setup
archive_previous_run "$PRD_FILE" "$PROGRESS_FILE" "$ARCHIVE_DIR" "$LAST_BRANCH_FILE"
track_branch "$PRD_FILE" "$LAST_BRANCH_FILE"
init_progress_file "$PROGRESS_FILE"

echo "Starting Ralph - Max iterations: $MAX_ITERATIONS"
echo "Using CLI: $CLI_TOOL (${MODEL:-default model})"

for i in $(seq 1 $MAX_ITERATIONS); do
	echo ""
	echo "═══════════════════════════════════════════════════════"
	echo "  Ralph Iteration $i of $MAX_ITERATIONS"
	echo "═══════════════════════════════════════════════════════"

	OUTPUT=$(execute_tool "$CLI_TOOL" "$MODEL" "$SHARE" "$PROMPT_FILE" "$CONFIG") || true

	if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
		echo ""
		echo "Ralph completed all tasks!"
		echo "Completed at iteration $i of $MAX_ITERATIONS"
		exit 0
	fi

	echo "Iteration $i complete. Continuing..."
	sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
exit 1
