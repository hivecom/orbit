#!/usr/bin/env bash

SESSION="orbit_dev"
tmux kill-session -t $SESSION 2>/dev/null
tmux new-session -d -s $SESSION "vp run platform#dev"
tmux split-window -h -t $SESSION "vp run dev"
tmux attach-session -t $SESSION
