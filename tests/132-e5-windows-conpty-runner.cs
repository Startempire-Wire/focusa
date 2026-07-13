using System;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

public static class Spec132ConPtyRunner
{
    [StructLayout(LayoutKind.Sequential)] struct COORD { public short X, Y; }
    [StructLayout(LayoutKind.Sequential)] struct SECURITY_ATTRIBUTES { public int nLength; public IntPtr lpSecurityDescriptor; public int bInheritHandle; }
    [StructLayout(LayoutKind.Sequential)] struct PROCESS_INFORMATION { public IntPtr hProcess, hThread; public int processId, threadId; }
    [StructLayout(LayoutKind.Sequential)] struct STARTUPINFO { public int cb; public IntPtr lpReserved, lpDesktop, lpTitle; public int dwX, dwY, dwXSize, dwYSize, dwXCountChars, dwYCountChars, dwFillAttribute, dwFlags; public short wShowWindow, cbReserved2; public IntPtr lpReserved2, hStdInput, hStdOutput, hStdError; }
    [StructLayout(LayoutKind.Sequential)] struct STARTUPINFOEX { public STARTUPINFO StartupInfo; public IntPtr lpAttributeList; }

    const uint PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE = 0x00020016;
    const uint EXTENDED_STARTUPINFO_PRESENT = 0x00080000;
    const uint CREATE_UNICODE_ENVIRONMENT = 0x00000400;
    const uint WAIT_OBJECT_0 = 0x00000000;
    const uint WAIT_TIMEOUT = 0x00000102;
    const uint CONPTY_TIMEOUT_MS = 60000;

    const uint HANDLE_FLAG_INHERIT = 0x00000001;
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool CreatePipe(out IntPtr read, out IntPtr write, ref SECURITY_ATTRIBUTES attrs, int size);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool SetHandleInformation(IntPtr handle, uint mask, uint flags);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll", SetLastError=true)] static extern int CreatePseudoConsole(COORD size, IntPtr input, IntPtr output, uint flags, out IntPtr pty);
    [DllImport("kernel32.dll", SetLastError=true)] static extern int ResizePseudoConsole(IntPtr pty, COORD size);
    [DllImport("kernel32.dll")] static extern void ClosePseudoConsole(IntPtr pty);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool InitializeProcThreadAttributeList(IntPtr list, int count, int flags, ref IntPtr size);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool UpdateProcThreadAttribute(IntPtr list, uint flags, IntPtr attribute, IntPtr value, IntPtr size, IntPtr previous, IntPtr returnSize);
    [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)] static extern bool CreateProcess(string app, StringBuilder command, IntPtr pa, IntPtr ta, bool inherit, uint flags, IntPtr env, string cwd, ref STARTUPINFOEX startup, out PROCESS_INFORMATION process);
    [DllImport("kernel32.dll")] static extern uint WaitForSingleObject(IntPtr handle, uint timeout);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool TerminateProcess(IntPtr handle, uint exitCode);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool GetExitCodeProcess(IntPtr handle, out uint code);

    static void Check(bool ok, string operation) { if (!ok) { var error = Marshal.GetLastWin32Error(); throw new Win32Exception(error, $"{operation} (Win32={error})"); } }
    static void Close(IntPtr h) { if (h != IntPtr.Zero && h != new IntPtr(-1)) CloseHandle(h); }

    public static string Run(string executable, string arguments, int width, int height, out int exitCode)
        => Run(executable, arguments, width, height, CONPTY_TIMEOUT_MS, out exitCode);

    public static string Run(string executable, string arguments, int width, int height, uint timeoutMs, out int exitCode)
    {
        IntPtr inRead, inWrite, outRead, outWrite;
        var pipeAttributes = new SECURITY_ATTRIBUTES {
            nLength = Marshal.SizeOf<SECURITY_ATTRIBUTES>(),
            bInheritHandle = 1,
        };
        Check(CreatePipe(out inRead, out inWrite, ref pipeAttributes, 0), "CreatePipe input");
        Check(CreatePipe(out outRead, out outWrite, ref pipeAttributes, 0), "CreatePipe output");
        Check(SetHandleInformation(inWrite, HANDLE_FLAG_INHERIT, 0), "SetHandleInformation input");
        Check(SetHandleInformation(outRead, HANDLE_FLAG_INHERIT, 0), "SetHandleInformation output");
        IntPtr pty = IntPtr.Zero, list = IntPtr.Zero, ptyValue = IntPtr.Zero;
        PROCESS_INFORMATION pi = default;
        try
        {
            var size = new COORD { X=(short)width, Y=(short)height };
            Check(CreatePseudoConsole(size, inRead, outWrite, 0, out pty) == 0, "CreatePseudoConsole");
            Close(inRead); Close(outWrite); inRead = outWrite = IntPtr.Zero;
            IntPtr bytes = IntPtr.Zero;
            InitializeProcThreadAttributeList(IntPtr.Zero, 1, 0, ref bytes);
            list = Marshal.AllocHGlobal(bytes);
            Check(InitializeProcThreadAttributeList(list, 1, 0, ref bytes), "InitializeProcThreadAttributeList");
            // PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE expects lpValue to point to
            // an HPCON-sized value. Passing HPCON itself as lpValue attaches an
            // invalid attribute and lets child output escape the capture pipe.
            ptyValue = Marshal.AllocHGlobal(IntPtr.Size);
            Marshal.WriteIntPtr(ptyValue, pty);
            Check(UpdateProcThreadAttribute(list, 0, new IntPtr(unchecked((long)PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE)), ptyValue, IntPtr.Size, IntPtr.Zero, IntPtr.Zero), "UpdateProcThreadAttribute");
            var startup = new STARTUPINFOEX {
                StartupInfo = new STARTUPINFO { cb = Marshal.SizeOf<STARTUPINFOEX>() },
                lpAttributeList = list,
            };
            var fullExecutable = Path.GetFullPath(executable);
            var command = new StringBuilder("\"" + fullExecutable.Replace("\"", "\\\"") + "\" " + arguments);
            var workingDirectory = Path.GetDirectoryName(fullExecutable);
            Check(CreateProcess(fullExecutable, command, IntPtr.Zero, IntPtr.Zero, false, EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT, IntPtr.Zero, workingDirectory, ref startup, out pi), "CreateProcess");
            Close(pi.hThread); pi.hThread = IntPtr.Zero;
            var output = new StringBuilder();
            using (var stream = new FileStream(new SafeFileHandle(outRead, false), FileAccess.Read, 4096, false))
            using (var reader = new StreamReader(stream, Encoding.UTF8))
            {
                var task = reader.ReadToEndAsync();
                var wait = WaitForSingleObject(pi.hProcess, timeoutMs);
                if (wait == WAIT_TIMEOUT)
                {
                    // Never leave a hosted runner waiting forever. Terminate
                    // only the process created by this fixture, then tear
                    // down the PTY so the output reader receives EOF.
                    TerminateProcess(pi.hProcess, 0xDEAD);
                    WaitForSingleObject(pi.hProcess, 5000);
                    Close(inWrite); inWrite = IntPtr.Zero;
                    ClosePseudoConsole(pty); pty = IntPtr.Zero;
                    throw new TimeoutException("ConPTY child exceeded 60 second runtime limit");
                }
                if (wait != WAIT_OBJECT_0)
                    throw new Win32Exception(Marshal.GetLastWin32Error(), "WaitForSingleObject");

                // The child has exited, but ConPTY output EOF is not
                // guaranteed until both the parent input writer and the
                // pseudo-console are closed. Do this before draining output.
                Close(inWrite); inWrite = IntPtr.Zero;
                ClosePseudoConsole(pty); pty = IntPtr.Zero;
                output.Append(task.GetAwaiter().GetResult());
            }
            Check(GetExitCodeProcess(pi.hProcess, out var code), "GetExitCodeProcess");
            exitCode = unchecked((int)code);
            return output.ToString();
        }
        finally
        {
            Close(pi.hThread); Close(pi.hProcess); Close(inRead); Close(inWrite); Close(outRead); Close(outWrite);
            if (pty != IntPtr.Zero) ClosePseudoConsole(pty);
            if (list != IntPtr.Zero) { Marshal.FreeHGlobal(list); }
            if (ptyValue != IntPtr.Zero) { Marshal.FreeHGlobal(ptyValue); }

        }
    }
}
