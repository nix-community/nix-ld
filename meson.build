project('meson-rust', ['rust', 'c'])

#e = executable('nix-ld',
#               [
#                 'src/x86_64-unknown-linux-gnu/syscalls.c',
#                 'src/x86_64-unknown-linux-gnu/breakpoint.s',
#                 'src/x86_64-unknown-linux-gnu/jmp_ld.s',
#                 'src/x86_64-unknown-linux-gnu/start.s'
#               ],
#               link_with : l,
#               install : true,
#               link_args : ['-nostartfiles', '-static', '-nodefaultlibs'])
static_library('mylib',
               'source.c',
               )
l = executable('nixld',
           ['src/lib.rs',
            'src/breakpoint.rs',
            'src/errno.rs',
            'src/exit.rs',
            'src/fd.rs',
            'src/lib.rs',
            'src/lossy.rs',
            'src/print.rs',
            'src/string.rs',
            'src/syscalls.rs',
            'src/unwind_resume.rs',
            'src/x86_64-unknown-linux-gnu/syscalls.c',
            'src/x86_64-unknown-linux-gnu/breakpoint.s',
            'src/x86_64-unknown-linux-gnu/jmp_ld.s',
            'src/x86_64-unknown-linux-gnu/start.s'
           ],
           link_with: l,
           rust_args : [
             '--edition=2018',
             '-C', 'link-args=-nostartfiles -static -fPIC',
             '-C', 'panic=abort'
           ])
           #, rust_crate_type : 'staticlib')
