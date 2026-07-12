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
    [StructLayout(LayoutKind.Sequential)] struct STARTUPINFOEX { public int cb; public IntPtr lpReserved, lpDesktop, lpTitle; public int dwX, dwY, dwXSize, dwYSize, dwXCountChars, dwYCountChars, dwFillAttribute, dwFlags; public short wShowWindow, cbReserved2; public IntPtr lpReserved2, hStdInput, hStdOutput, hStdError, lpAttributeList; }

    const uint PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE = 0x00020016;
    const uint EXTENDED_STARTUPINFO_PRESENT = 0x00080000;
    const uint CREATE_UNICODE_ENVIRONMENT = 0x00000400;
    const uint INFINITE = 0xffffffff;

    [DllImport("kernel32.dll", SetLastError=true)] static extern bool CreatePipe(out IntPtr read, out IntPtr write, IntPtr attrs, int size);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll", SetLastError=true)] static extern int CreatePseudoConsole(COORD size, IntPtr input, IntPtr output, uint flags, out IntPtr pty);
    [DllImport("kernel32.dll", SetLastError=true)] static extern int ResizePseudoConsole(IntPtr pty, COORD size);
    [DllImport("kernel32.dll")] static extern void ClosePseudoConsole(IntPtr pty);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool InitializeProcThreadAttributeList(IntPtr list, int count, int flags, ref IntPtr size);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool UpdateProcThreadAttribute(IntPtr list, uint flags, IntPtr attribute, IntPtr value, IntPtr size, IntPtr previous, IntPtr returnSize);
    [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)] static extern bool CreateProcess(string app, StringBuilder command, IntPtr pa, IntPtr ta, bool inherit, uint flags, IntPtr env, string cwd, ref STARTUPINFOEX startup, out PROCESS_INFORMATION process);
    [DllImport("kernel32.dll")] static extern uint WaitForSingleObject(IntPtr handle, uint timeout);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool GetExitCodeProcess(IntPtr handle, out uint code);

    static void Check(bool ok, string operation) { if (!ok) throw new Win32Exception(Marshal.GetLastWin32Error(), operation); }
    static void Close(IntPtr h) { if (h != IntPtr.Zero && h != new IntPtr(-1)) CloseHandle(h); }

    public static string Run(string executable, string arguments, int width, int height, out int exitCode)
    {
        IntPtr inRead, inWrite, outRead, outWrite;
        Check(CreatePipe(out inRead, out inWrite, IntPtr.Zero, 0), "CreatePipe input");
        Check(CreatePipe(out outRead, out outWrite, IntPtr.Zero, 0), "CreatePipe output");
        IntPtr pty = IntPtr.Zero, list = IntPtr.Zero, processAttribute = IntPtr.Zero;
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
            processAttribute = Marshal.AllocHGlobal(IntPtr.Size);
            Marshal.WriteIntPtr(processAttribute, pty);
            Check(UpdateProcThreadAttribute(list, 0, new IntPtr(unchecked((long)PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE)), processAttribute, IntPtr.Size, IntPtr.Zero, IntPtr.Zero), "UpdateProcThreadAttribute");
            var startup = new STARTUPINFOEX { cb=Marshal.SizeOf<STARTUPINFOEX>(), lpAttributeList=list };
            var command = new StringBuilder("\"" + executable.Replace("\"", "\\\"") + "\" " + arguments);
            Check(CreateProcess(null, command, IntPtr.Zero, IntPtr.Zero, false, EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT, IntPtr.Zero, null, ref startup, out pi), "CreateProcess");
            Close(pi.hThread); pi.hThread = IntPtr.Zero;
            var output = new StringBuilder();
            using (var stream = new FileStream(new SafeFileHandle(outRead, false), FileAccess.Read, 4096, false))
            using (var reader = new StreamReader(stream, Encoding.UTF8))
            {
                var task = reader.ReadToEndAsync();
                WaitForSingleObject(pi.hProcess, INFINITE);
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
            if (processAttribute != IntPtr.Zero) Marshal.FreeHGlobal(processAttribute);
        }
    }
}
