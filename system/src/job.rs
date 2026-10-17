//! Ending yt-dlp together with the programs it starts.
//!
//! yt-dlp for Windows is a program that unpacks itself and starts a second
//! copy, which may start ffmpeg; ending only the first leaves the others
//! running. So each yt-dlp runs in a Windows job object, and ending the job
//! ends them all. Closing the job's last handle ends them too, so nothing is
//! left behind when keepclip closes in the middle of a download.

/// A group of processes that end together. Elsewhere than on Windows, only
/// the process itself is ended.
pub struct Job {
    #[cfg(windows)]
    handle: windows::Win32::Foundation::HANDLE,
    #[cfg(not(windows))]
    pid: u32,
}

// SAFETY: a job handle may be used and closed from any thread.
#[cfg(windows)]
unsafe impl Send for Job {}
#[cfg(windows)]
unsafe impl Sync for Job {}

#[cfg(windows)]
impl Job {
    /// Puts `child`, and every process it starts from now on, in a new job.
    /// `None` when Windows refuses; then the child runs on its own.
    pub fn for_child(child: &std::process::Child) -> Option<Job> {
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject,
        };
        use windows::core::PCWSTR;

        // SAFETY: no name and default security; the handle is owned by the
        // returned `Job`, which closes it.
        let handle = unsafe { CreateJobObjectW(None, PCWSTR::null()) }.ok()?;
        let job = Job { handle };
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: the structure outlives the call, and the length is its own.
        unsafe {
            SetInformationJobObject(
                job.handle,
                JobObjectExtendedLimitInformation,
                std::ptr::from_ref(&limits).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        }
        .ok()?;
        // yt-dlp takes a moment to unpack itself before it starts its second
        // copy, so it is in the job before that copy exists.
        // SAFETY: both handles are valid: the child's lives as long as `child`.
        unsafe { AssignProcessToJobObject(job.handle, HANDLE(child.as_raw_handle())) }.ok()?;
        Some(job)
    }

    /// Ends every process in the job.
    pub fn kill(&self) {
        use windows::Win32::System::JobObjects::TerminateJobObject;
        // SAFETY: the handle is valid until drop. Nothing to do if it fails:
        // the processes have ended already.
        let _ = unsafe { TerminateJobObject(self.handle, 1) };
    }
}

#[cfg(windows)]
impl Drop for Job {
    fn drop(&mut self) {
        use windows::Win32::Foundation::CloseHandle;
        // SAFETY: the handle is owned by this `Job` and closed once.
        let _ = unsafe { CloseHandle(self.handle) };
    }
}

#[cfg(not(windows))]
impl Job {
    pub fn for_child(child: &std::process::Child) -> Option<Job> {
        Some(Job { pid: child.id() })
    }

    pub fn kill(&self) {
        let _ = std::process::Command::new("kill")
            .arg(self.pid.to_string())
            .status();
    }
}
