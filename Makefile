PREFIX ?= /usr
SYSCONFDIR ?= /etc
LOCALSTATEDIR ?= /var
BINDIR = $(DESTDIR)$(PREFIX)/bin
DATADIR = $(DESTDIR)$(PREFIX)/share/meowtrics
PLASMOIDDIR = $(DESTDIR)$(PREFIX)/share/plasma/plasmoids/com.ra-yavuz.meowtrics
USERUNITDIR = $(DESTDIR)$(PREFIX)/lib/systemd/user
DOCDIR = $(DESTDIR)$(PREFIX)/share/doc/meowtrics

CARGO ?= cargo
TARGET_BIN = target/release/meowtrics

.PHONY: all build install uninstall lint check deb plasmoid clean

all: build

build:
	$(CARGO) build --release

install: build
	install -d $(BINDIR) $(DATADIR) $(PLASMOIDDIR) $(USERUNITDIR) $(DOCDIR)
	install -m 0755 $(TARGET_BIN)                    $(BINDIR)/meowtrics
	install -m 0644 data/messages.json               $(DATADIR)/messages.json
	install -m 0644 systemd/meowtrics.service        $(USERUNITDIR)/meowtrics.service
	install -m 0644 README.md                        $(DOCDIR)/README.md
	install -m 0644 LICENSE                          $(DOCDIR)/LICENSE
	install -m 0644 LICENSING.md                     $(DOCDIR)/LICENSING.md
	cp -r plasmoid/* $(PLASMOIDDIR)/

uninstall:
	rm -f $(BINDIR)/meowtrics
	rm -rf $(DATADIR) $(PLASMOIDDIR) $(DOCDIR)
	rm -f $(USERUNITDIR)/meowtrics.service

deb:
	bash scripts/build-deb.sh

plasmoid:
	bash scripts/build-plasmoid.sh

lint:
	$(CARGO) clippy --all-targets -- -D warnings
	$(CARGO) fmt --all -- --check
	shellcheck scripts/*.sh debian/postinst debian/postrm 2>/dev/null || true

check: lint
	$(CARGO) test

clean:
	$(CARGO) clean
	rm -rf dist
