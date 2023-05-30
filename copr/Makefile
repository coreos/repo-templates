# Maintained in https://github.com/coreos/repo-templates
# Do not edit downstream.

.PHONY: srpm
srpm:
	dnf install -y git rpm-build rpmdevtools {% if crate %}rust-packaging{% endif %}
	# similar to https://github.com/actions/checkout/issues/760, but for COPR
	git config --global --add safe.directory '*'
	curl -LOf https://src.fedoraproject.org/rpms/{{ fedora_package }}/raw/rawhide/f/{{ fedora_package }}.spec
	version=$$(git describe --always --tags | sed -e 's,-,\.,g' -e 's,^v,,'); \
	git archive --format=tar --prefix={{ git_repo }}-$$version/ HEAD | gzip > {{ git_repo }}-$$version.{% if crate %}crate{% else %}tar.gz{% endif %}; \
	sed -ie "s,^Version:.*,Version: $$version," {{ fedora_package }}.spec
	sed -ie 's/^Patch/# Patch/g' {{ fedora_package }}.spec  # we don't want any downstream patches
{%- if crate %}
	sed -ie 's/^Source1/# Source1/g' {{ fedora_package }}.spec  # we don't vendor
{%- endif %}
	spectool -g {{ fedora_package }}.spec  # download any remaining sources (e.g. coreos-installer-dracut)
	rpmbuild -bs --define "_sourcedir ${PWD}" --define "_specdir ${PWD}" --define "_builddir ${PWD}" --define "_srcrpmdir ${PWD}" --define "_rpmdir ${PWD}" --define "_buildrootdir ${PWD}/.build" {{ fedora_package }}.spec
	mv *.src.rpm $$outdir
