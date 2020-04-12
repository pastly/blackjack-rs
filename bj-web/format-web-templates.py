#!/usr/bin/env python3
from argparse import ArgumentParser, ArgumentDefaultsHelpFormatter
from glob import iglob
import os
import logging
from subprocess import run


BIN_EXTS = [
    '.wasm',
    '.png',
]
VERSION_STR = None

logging.basicConfig(level=logging.INFO)


def abspath(p):
    op = os.path
    return op.abspath(op.expanduser(op.expandvars(p)))


def find_files(dname):
    for name in iglob(f'{dname}/**', recursive=True):
        if os.path.isfile(name):
            yield name


def replace_prefix(s, bad_prefix, new_prefix):
    assert s.startswith(bad_prefix)
    return new_prefix + s[len(bad_prefix):]


def is_binary_file(fname):
    ext = os.path.splitext(fname)[1]
    return ext in BIN_EXTS


def get_version_str():
    global VERSION_STR
    if VERSION_STR is not None:
        return VERSION_STR
    proc = run(
        'git rev-parse --short HEAD'.split(),
        text=True, capture_output=True)
    commit = proc.stdout.strip()
    proc = run(
        'git show -s --format=%cd --date=format:%Y-%m-%d'.split() + [commit],
        text=True, capture_output=True)
    date = proc.stdout.strip()
    VERSION_STR = f'{date} ({commit})'
    return VERSION_STR


def get_google_shit():
    return '''
  <!-- Global site tag (gtag.js) - Google Analytics -->
  <script async src="https://www.googletagmanager.com/gtag/js?id=UA-160379782-1"></script>
  <script>
    window.dataLayer = window.dataLayer || [];
    function gtag(){dataLayer.push(arguments);}
    gtag('js', new Date());

    gtag('config', 'UA-160379782-1');
  </script>
  <script data-ad-client="ca-pub-3834375319956666" async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js"></script>
'''

def get_nav_bar():
    return '''
<nav>
<a href='index.html'><img alt="BJ logo" id=logo src="static/logo.png" /></a>
<ul>
<li><a href='index.html'>Game</a></li>
<li><a href='custom-card.html'>Customize</a></li>
</ul>
</nav>
'''

def main(args):
    for in_fname in find_files(args.input):
        out_fname = replace_prefix(in_fname, args.input, args.output)
        dirname = os.path.dirname(out_fname)
        logging.debug(f'Making sure {dirname} exists')
        os.makedirs(dirname, exist_ok=True)
        rmode = 'rb' if is_binary_file(in_fname) else 'rt'
        wmode = 'wb' if is_binary_file(out_fname) else 'wt'
        with open(in_fname, rmode) as ifd, open(out_fname, wmode) as ofd:
            if is_binary_file(in_fname):
                logging.info(f'Considering {in_fname} a binary file')
                ofd.write(ifd.read())
                continue
            logging.info(f'Considering {in_fname} a text file')
            s = ifd.read()
            s = s.replace('<!-- BJ_TMPL_NAV_BAR -->', get_nav_bar())
            s = s.replace('<!-- BJ_TMPL_VERSION -->', get_version_str())
            s = s.replace('<!-- GOOGLE_SHIT -->', get_google_shit())
            ofd.write(s)


if __name__ == '__main__':
    p = ArgumentParser(formatter_class=ArgumentDefaultsHelpFormatter)
    p.add_argument(
        '-i', '--input', type=str, default='www', help='Input directory')
    p.add_argument(
        '-o', '--output', type=str, default='www-out', help='Output directory')
    args = p.parse_args()
    args.input = abspath(args.input)
    args.output = abspath(args.output)
    assert os.path.isdir(args.input), f'{args.input} does not exist'
    if os.path.isdir(args.output):
        logging.warning(
            f'{args.output} exists. Files inside will be overwritten')
    exit(main(args))
