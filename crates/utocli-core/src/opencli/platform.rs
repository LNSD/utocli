//! Platform and architecture entities.

/// Declares OS and architecture support.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Platform {
    /// The platform operating system name.
    pub name: PlatformName,

    /// The supported architectures for this platform.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architectures: Option<Vec<Architecture>>,
}

impl Platform {
    /// Creates a new `Platform` with the given name.
    pub fn new(name: PlatformName) -> Self {
        Self {
            name,
            architectures: None,
        }
    }

    /// Sets the supported architectures for this platform.
    pub fn architectures(mut self, architectures: Vec<Architecture>) -> Self {
        self.architectures = Some(architectures);
        self
    }
}

/// Platform operating system names.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlatformName {
    /// Microsoft Windows
    Windows,
    /// Apple macOS
    Macos,
    /// Apple Darwin (legacy name for macOS)
    Darwin,
    /// Apple iOS
    Ios,
    /// Linux
    Linux,
    /// Google Android
    Android,
    /// FreeBSD
    Freebsd,
    /// DragonFly BSD
    Dragonfly,
    /// OpenBSD
    Openbsd,
    /// NetBSD
    Netbsd,
    /// IBM AIX
    Aix,
    /// Oracle Solaris
    Solaris,
}

/// CPU architecture types.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    /// AMD64 / x86_64 (64-bit)
    Amd64,
    /// x86_64 (64-bit, alternative naming)
    #[serde(rename = "x86_64")]
    X86_64,
    /// Intel 386 (32-bit)
    #[serde(rename = "386")]
    I386,
    /// x86 (32-bit)
    X86,
    /// ARM 64-bit
    Arm64,
    /// AArch64 (ARM 64-bit, alternative naming)
    Aarch64,
    /// ARM (generic)
    Arm,
    /// ARMv5TE
    Armv5te,
    /// ARMv7
    Armv7,
    /// Thumb v7
    Thumbv7,
    /// PowerPC 64-bit
    Ppc64,
    /// PowerPC 64-bit Little Endian
    Ppc64le,
    /// PowerPC (generic)
    Powerpc,
    /// PowerPC 64-bit (alternative naming)
    Powerpc64,
    /// PowerPC 64-bit Little Endian (alternative naming)
    Powerpc64le,
    /// MIPS
    Mips,
    /// MIPS Little Endian
    Mipsel,
    /// MIPS 64-bit
    Mips64,
    /// MIPS 64-bit Little Endian
    Mips64el,
    /// IBM System z (s390x)
    S390x,
    /// RISC-V 64-bit
    Riscv64,
    /// RISC-V 32-bit
    Riscv32,
    /// WebAssembly 32-bit
    Wasm32,
    /// WebAssembly 64-bit
    Wasm64,
    /// SPARC 64-bit
    Sparc64,
    /// Qualcomm Hexagon
    Hexagon,
    /// LoongArch 64-bit
    Loongarch64,
}
