.PHONY: diff
diff: tmpl8
	tmpl8/target/debug/tmpl8 diff | less -R

.PHONY: output
output: tmpl8
	tmpl8/target/debug/tmpl8 render output

.PHONY: tmpl8
tmpl8:
	cd tmpl8 && cargo build
