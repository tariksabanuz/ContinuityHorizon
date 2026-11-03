import re, pathlib, subprocess, sys
here = pathlib.Path(__file__).parent
b = here.parent / 'mixins' / 'connected_textures'
t = (here / 'template.rs').read_text().replace('//CTM_PATH//', str((b / 'ctm.rs').resolve()))
for tag, f, name in [('MESH','mesh.rs','mesh'),('LISTED','listed.rs','listed'),('SETTINGS','settings.rs','settings'),('SETTINGS_CLICK','settings_click.rs','settings_click'),('FRAME','frame.rs','frame')]:
    src = (b / f).read_text()
    src = re.sub(r'\bfn connected_textures_\w+\(', f'fn __{name}(', src, count=1)
    src = src.replace('crate::native_connected_textures', 'crate::horizon_client_render::native_connected_textures')
    t = t.replace(f'//{tag}//', src)
(here / 'check.rs').write_text(t)
r = subprocess.run(['rustc', '--edition', '2024', '--crate-type', 'lib', '--crate-name', 'horizon_client', 'check.rs', '-o', '/tmp/ct_check.rlib'], cwd=here, capture_output=True, text=True)
print(r.stderr[-6000:]); sys.exit(r.returncode)
