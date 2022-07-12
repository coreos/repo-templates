FORK_ARGS = \
	--fork-regex /coreos/ \
	--fork-replacement /coreosbot-releng/ \
	--fork-branch repo-templates

.PHONY: diff
diff: tmpl8
	tmpl8/target/debug/tmpl8 diff $(FORK_ARGS) | less -R

.PHONY: output
output: tmpl8
	tmpl8/target/debug/tmpl8 render output

# Force sync of downstream repo cache
.PHONY: sync
sync: tmpl8
	tmpl8/target/debug/tmpl8 update-cache $(FORK_ARGS)

.PHONY: tmpl8
tmpl8:
	cd tmpl8 && cargo build
