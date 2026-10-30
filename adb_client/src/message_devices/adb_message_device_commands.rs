/// Implement [`ADBDeviceExt`](crate::ADBDeviceExt) for a device wrapping an `ADBMessageDevice` in its `inner`
/// field.
macro_rules! impl_message_device_commands {
    ($device:ty) => {
        impl $crate::ADBDeviceExt for $device {
            #[inline]
            fn shell_command(
                &self,
                command: &dyn AsRef<str>,
                stdout: Option<&mut dyn std::io::Write>,
                stderr: Option<&mut dyn std::io::Write>,
            ) -> $crate::Result<Option<u8>> {
                self.inner.shell_command(command, stdout, stderr)
            }

            #[inline]
            fn shell(
                &self,
                reader: Box<dyn std::io::Read + Send>,
                writer: Box<dyn std::io::Write + Send>,
            ) -> $crate::Result<()> {
                self.inner.shell(reader, writer)
            }

            #[inline]
            fn exec(
                &self,
                command: &str,
                reader: Box<dyn std::io::Read + Send>,
                writer: Box<dyn std::io::Write + Send>,
            ) -> $crate::Result<()> {
                self.inner.exec(command, reader, writer)
            }

            #[inline]
            fn stat(
                &self,
                remote_path: &dyn AsRef<str>,
            ) -> $crate::Result<$crate::AdbStatResponse> {
                self.inner.stat(remote_path)
            }

            #[inline]
            fn pull(
                &self,
                source: &dyn AsRef<str>,
                output: &mut dyn std::io::Write,
            ) -> $crate::Result<()> {
                self.inner.pull(source, output)
            }

            #[inline]
            fn push(
                &self,
                stream: &mut dyn std::io::Read,
                path: &dyn AsRef<str>,
            ) -> $crate::Result<()> {
                self.inner.push(stream, path)
            }

            #[inline]
            fn list(&self, path: &dyn AsRef<str>) -> $crate::Result<Vec<$crate::ADBListItemType>> {
                self.inner.list(path)
            }

            #[inline]
            fn reboot(&self, reboot_type: $crate::RebootType) -> $crate::Result<()> {
                self.inner.reboot(reboot_type)
            }

            #[inline]
            fn remount(&self) -> $crate::Result<Vec<$crate::RemountInfo>> {
                self.inner.remount()
            }

            #[inline]
            fn root(&self) -> $crate::Result<()> {
                self.inner.root()
            }

            #[inline]
            fn install(
                &self,
                apk_path: &dyn AsRef<std::path::Path>,
                user: Option<&str>,
            ) -> $crate::Result<()> {
                self.inner.install(apk_path, user)
            }

            #[inline]
            fn uninstall(
                &self,
                package: &dyn AsRef<str>,
                user: Option<&str>,
            ) -> $crate::Result<()> {
                self.inner.uninstall(package, user)
            }

            #[inline]
            fn enable_verity(&self) -> $crate::Result<()> {
                self.inner.enable_verity()
            }

            #[inline]
            fn disable_verity(&self) -> $crate::Result<()> {
                self.inner.disable_verity()
            }

            #[inline]
            #[cfg(feature = "framebuffer")]
            fn framebuffer_inner(
                &self,
            ) -> $crate::Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
                self.inner.framebuffer_inner()
            }
        }
    };
}

pub(crate) use impl_message_device_commands;
