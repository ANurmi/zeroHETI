.PHONY: ips
ips:
	bender update

.PHONY: vlint
vlint:
	$(MAKE) -C verilator lint

.PHONY: verilate
verilate:
	$(MAKE) -C verilator verilate

.PHONY: simv
simv:
	$(MAKE) -C verilator simv

.PHONY: clean_build
clean_build:
	rm -fr build

.PHONY: clean_ips
clean_ips:
	rm -fr .bender

clean_all: clean_build clean_ips
