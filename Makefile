SHELL := /usr/bin/env bash

RELEASE ?= $(shell date -u +%Y%m%d%H%M%S)

.PHONY: build build-backend build-frontend package deploy rollback debug

build: build-backend build-frontend

build-backend:
	./scripts/build_backend.sh

build-frontend:
	./scripts/build_frontend.sh

package:
	./scripts/package_release.sh $(RELEASE)

deploy:
	./scripts/deploy_vps.sh $(RELEASE)

rollback:
	./scripts/rollback_vps.sh $(RELEASE)

debug:
	./scripts/debug_local.sh
