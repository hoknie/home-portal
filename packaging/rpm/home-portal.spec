Name:           home-portal
Version:        %{portal_version}
Release:        1
Summary:        A start page for a home network that knows which services are up
License:        Apache-2.0
URL:            https://github.com/hoknie/home-portal
AutoReqProv:    no
Requires:       ca-certificates

%global debug_package %{nil}
%global __os_install_post %{nil}

%description
home-portal probes home services over HTTP, TCP and ICMP, keeps their history, shows a
home page arranged in the browser, publishes services through Caddy and runs automations.
One static binary and its interface in /usr/share/home-portal/web, run by systemd as the user home-portal.

%install
cp -a %{_sourcedir}/. %{buildroot}/

%files
/usr/bin/home-portal
/usr/lib/home-portal
/usr/lib/systemd/system/home-portal.service
/usr/lib/sysusers.d/home-portal.conf
/usr/lib/tmpfiles.d/home-portal.conf
/usr/share/home-portal
%doc /usr/share/doc/home-portal

%post
if [ "$1" -eq 1 ]; then
    /usr/lib/home-portal/after-install first
else
    /usr/lib/home-portal/after-install upgrade
fi

%preun
if [ "$1" -eq 0 ]; then
    /usr/lib/home-portal/before-remove
fi

%postun
if [ -d /run/systemd/system ]; then
    systemctl daemon-reload || true
fi
