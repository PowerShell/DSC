Name:           dsc
Version:        VERSION_PLACEHOLDER
Release:        1
Summary:        Desired State Configuration v3
License:        MIT
URL:            https://github.com/PowerShell/DSC
BuildArch:      ARCH_PLACEHOLDER

%description
DSCv3 is the latest iteration of Microsoft's Desired State Configuration platform.
DSCv3 is an open source command line application that abstracts the management of
software components declaratively and idempotently. DSCv3 runs on Linux, macOS,
and Windows without any external dependencies.

%prep
# No prep needed - files are already built

%build
# No build needed - binary is already compiled

%install
# Create installation directories
mkdir -p $RPM_BUILD_ROOT/opt/dsc
mkdir -p $RPM_BUILD_ROOT/usr/bin

# Copy all files from the source directory
cp -r $RPM_SOURCE_DIR/dsc_files/* $RPM_BUILD_ROOT/opt/dsc/

# Create symlink to make dsc available in PATH
ln -s /opt/dsc/dsc $RPM_BUILD_ROOT/usr/bin/dsc

%files
/opt/dsc/*
/usr/bin/dsc

%changelog
* Wed Oct 22 2025 Microsoft Corporation
- Initial RPM package release
