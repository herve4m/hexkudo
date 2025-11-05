#!/bin/bash

rm -rf repo .flatpak-builder builddir /tmp/hexkudo.flatpak
flatpak uninstall -y io.github.herve4m.Hexkudo.Devel

#flatpak remote-add --user --if-not-exists gnome-nightly https://nightly.gnome.org/gnome-nightly.flatpakrepo
flatpak install -y flathub org.freedesktop.Sdk.Extension.rust-stable//25.08
flatpak install -y flathub org.gnome.Sdk//49
flatpak install -y flathub org.gnome.Platform//49
#flatpak-builder --force-clean --user --install-deps-from=gnome-nightly --repo=repo --install builddir io.github.herve4m.Hexkudo.yaml
flatpak-builder --force-clean --user --repo=repo --install builddir io.github.herve4m.Hexkudo.Devel.yaml
flatpak build-bundle repo /tmp/hexkudo.flatpak io.github.herve4m.Hexkudo.Devel
rm -rf repo .flatpak-builder builddir
flatpak run io.github.herve4m.Hexkudo.Devel
