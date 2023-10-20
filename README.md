# pmap-dotnet

This tool helps to understand the memory used parse the output of the [`pmap`](https://www.man7.org/linux/man-pages/man1/pmap.1.html) command to get and overview, which memory pages are used and to practice rust😉

## How to use

1. Run your dotnet application under linux (e.g. in docker)
2. Get the pid of your process `ps aux  grep | <appname>`
3. Run pmap `pmap -XX -p -q <pid>` > appname_ptrace
4. Run pmap-dotnet `cargo run -- --pmap-output="<FullPathTo_appname_pmap"`

## Results

### Overview of Categories

This section tries to group the memory pages into categories to better understand the memory usage.

```output
|----------------------------------------------------------|------------|-----------------|
| Category                                                 | Size [KiB] | #Memory Pages   |
|----------------------------------------------------------|------------|-----------------|
| Anonymous                                                |  274807572 |             370 |
| JIT Code                                                 |      41352 |            3538 |
| Microsoft.CodeAnalysis.CSharp.dll                        |      36068 |               5 |
| libicudata.so.72.1                                       |      30536 |               5 |
| Microsoft.CodeAnalysis.VisualBasic.dll                   |      27812 |               5 |
| System.Private.CoreLib.dll                               |      22316 |               5 |
| Microsoft.CodeAnalysis.dll                               |      14932 |               5 |
| libcoreclr.so                                            |       7052 |               6 |
| libcrypto.so.3                                           |       4600 |               5 |
| System.Security.Cryptography.dll                         |       4276 |               5 |
| libclrjit.so                                             |       3396 |               4 |
| libicui18n.so.72.1                                       |       3236 |               5 |
| [heap]                                                   |       2580 |               1 |
| Microsoft.CodeAnalysis.NetAnalyzers.dll                  |       2264 |               1 |
| System.Reflection.Metadata.dll                           |       2180 |               5 |
| libstdc++.so.6.0.30                                      |       2140 |               5 |
| libicuuc.so.72.1                                         |       2032 |               5 |
| System.Text.RegularExpressions.dll                       |       1900 |               5 |
| libc.so.6                                                |       1872 |               5 |
| System.Net.Sockets.dll                                   |       1188 |               5 |
| System.Collections.Immutable.dll                         |       1172 |               5 |
| System.Linq.dll                                          |        964 |               5 |
| libm.so.6                                                |        892 |               5 |
| libssl.so.3                                              |        676 |               5 |
| System.Runtime.Numerics.dll                              |        620 |               5 |
| System.Collections.dll                                   |        516 |               5 |
| System.Collections.Concurrent.dll                        |        476 |               5 |
| System.Net.Primitives.dll                                |        440 |               5 |
| VBCSCompiler.dll                                         |        436 |               5 |
| Microsoft.CodeAnalysis.CSharp.resources.dll              |        432 |               1 |
| System.Console.dll                                       |        420 |               5 |
| libhostfxr.so                                            |        412 |               4 |
| libhostpolicy.so                                         |        372 |               4 |
| System.Memory.dll                                        |        296 |               5 |
| System.Text.RegularExpressions.Generator.dll             |        288 |               1 |
| Microsoft.CodeAnalysis.NetAnalyzers.resources.dl         |        284 |               1 |
| System.Threading.Tasks.Parallel.dll                      |        260 |               5 |
| System.IO.Pipes.dll                                      |        260 |               5 |
| Microsoft.Interop.SourceGeneration.dll                   |        228 |               1 |
| ld-linux-x86-64.so.2                                     |        208 |               5 |
| System.Collections.Specialized.dll                       |        188 |               5 |
| Microsoft.Interop.LibraryImportGenerator.dll             |        184 |               1 |
| libSystem.Security.Cryptography.Native.OpenSsl.so        |        172 |               4 |
| System.IO.MemoryMappedFiles.dll                          |        164 |               5 |
| System.Text.Json.SourceGeneration.dll                    |        148 |               1 |
| System.Threading.dll                                     |        148 |               5 |
| dotnet                                                   |        144 |               4 |
| Microsoft.Interop.JavaScript.JSImportGenerator.dll       |        132 |               1 |
| [stack]                                                  |        132 |               1 |
| libgcc_s.so.1                                            |        128 |               5 |
| Microsoft.CodeAnalysis.CSharp.NetAnalyzers.dll           |        108 |               1 |
| System.Runtime.InteropServices.dll                       |        100 |               5 |
| libSystem.Native.so                                      |         96 |               4 |
| netstandard.dll                                          |         92 |               1 |
| Microsoft.Interop.LibraryImportGenerator.resourc         |         40 |               1 |
| System.Runtime.dll                                       |         32 |               1 |
| librt.so.1                                               |         20 |               5 |
| libdl.so.2                                               |         20 |               5 |
| libpthread.so.0                                          |         20 |               5 |
| [vvar]                                                   |         16 |               1 |
| System.Text.RegularExpressions.Generator.resourc         |         12 |               1 |
| System.Runtime.CompilerServices.Unsafe.dll               |          8 |               1 |
| System.Globalization.dll                                 |          8 |               1 |
| System.Security.Cryptography.Algorithms.dll              |          8 |               1 |
| System.Reflection.Primitives.dll                         |          8 |               1 |
| System.Reflection.Emit.ILGeneration.dll                  |          8 |               1 |
| System.Reflection.Emit.Lightweight.dll                   |          8 |               1 |
| System.Text.Encoding.Extensions.dll                      |          8 |               1 |
| System.Security.Cryptography.Primitives.dll              |          8 |               1 |
| System.Threading.Thread.dll                              |          8 |               1 |
| System.Threading.ThreadPool.dll                          |          8 |               1 |
| System.Diagnostics.Tracing.dll                           |          8 |               1 |
| Microsoft.Win32.Primitives.dll                           |          8 |               1 |
| System.Runtime.Loader.dll                                |          8 |               1 |
| [vdso]                                                   |          8 |               1 |
| 63a4afb306844d7b920b57fd377206a7                         |          4 |               1 |
|----------------------------------------------------------|------------|-----------------|
|                                                          |  275031168 |            4149 |
|----------------------------------------------------------|------------|-----------------|
```

### Overview of all memory pages bigger than 10 MiB

```output
|--------------|------------|--------------------------------|--------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
|   Address    | Size [KiB] |          Mapping Kind          |          Permissions           | VM Flags                                                                                                                                               |
|--------------|------------|--------------------------------|--------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
| 7f6f744e1000 |  267775100 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6ecb494000 |    2108848 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7faf68990000 |    1972672 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6ebf4ef000 |     130460 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6eb74f0000 |     130456 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6e44021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e4c021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e58021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e60021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e6c021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e78021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e8c021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e90021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e94021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e98021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e9c021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6ea0021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6ea4021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6ea8021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6eac021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6eb0021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7faf4c021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7faf54021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7faf58021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7faf5c021000 |      65404 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e8002c000 |      65360 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e7c02f000 |      65348 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e68047000 |      65252 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6ec74c3000 |      65224 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6e74063000 |      65140 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e5c064000 |      65136 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e7008e000 |      64968 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e883ba000 |      61720 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6e545f7000 |      59428 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Swap Space - Soft Dirty                                                                                           |
| 7f6f4e001000 |      32764 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f50001000 |      32764 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f66001000 |      32764 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f68001000 |      32764 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f6c001000 |      32764 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f4c011000 |      32700 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f52011000 |      32700 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f54011000 |      32700 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f56011000 |      32700 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f58011000 |      32700 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f5a011000 |      32700 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f70019000 |      32668 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f64030000 |      32576 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f62101000 |      31740 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f6a101000 |      31740 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6f6e201000 |      30716 | Anonymous Private              | Private                        | May Read - May Write - May Execute - Not Include In Core Dump - Soft Dirty                                                                             |
| 7f6eb5602000 |      30520 | libicudata.so.72.1             | Read - Private                 | Readable - May Read - May Write - May Execute - Soft Dirty                                                                                             |
| 7f6e65e00000 |      18028 | Microsoft.CodeAnalysis.CSharp.dll | Read - Share                   | Readable - May Read - May Execute - May Share - Soft Dirty                                                                                          |
| 7faf640f0000 |      14540 | Microsoft.CodeAnalysis.CSharp.dll | Read - Execute - Private       | Readable - Executable - May Read - May Write - May Execute - Soft Dirty                                                                             |
| 7f6e65000000 |      13900 | Microsoft.CodeAnalysis.VisualBasic.dll | Read - Share                   | Readable - May Read - May Execute - May Share - Soft Dirty                                                                                     |
| 7faf65370000 |      11216 | Microsoft.CodeAnalysis.VisualBasic.dll | Read - Execute - Private       | Readable - Executable - May Read - May Write - May Execute - Soft Dirty                                                                        |
| 7faf51a00000 |      11152 | System.Private.CoreLib.dll     | Read - Share                   | Readable - May Read - May Execute - May Share - Soft Dirty                                                                                             |
|--------------|------------|--------------------------------|--------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
```

### Estimated Threads

```output
Potential Number of Threads Stacks: 12 (Total: 98304 KiB)
```

## Configuration

| Name | Optional | Default | Usage |
|---|---|---|---|
| pmap-output | no | n/a | Path to the output file generated by pmap command |
| application-folder | yes | /app | Path to own application (e.g. within the container) |

## Background Knowledge

Linux store the actual allocated memory pages for a process under `/proc/<pid>/maps` and an more detailed overview under `proc/<pid>/smaps` (for more details please see [proc man page](https://man7.org/linux/man-pages/man5/proc.5.html)). The [pmap](https://man7.org/linux/man-pages/man5/proc.5.html) command prints the output of smaps in an easier to parse format.

pmap parameter used:

* `-XX` (only available under linux) to include all available kernel information
* `-p` to show full paths for file based mappings
* `-q` to not print header (explaining the different columns)
