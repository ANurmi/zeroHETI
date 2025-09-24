.PHONY: ips
ips:
	bender update

.PHONY: vlint
vlint:
	$(MAKE) -C verilator lint

.PHONY: clean_ips
clean_ips:
	rm -fr .bender
