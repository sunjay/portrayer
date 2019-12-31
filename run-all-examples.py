#!/usr/bin/env python3

import sys
import subprocess

from glob import glob
from os.path import basename, splitext
from subprocess import CalledProcessError
from shlex import quote

def main():
    for path in glob("examples/*.rs"):
        print("=============================================")
        print()

        example_name = splitext(basename(path))[0]
        print("Running example:", example_name)

        env = {"RUST_BACKTRACE": "1"}
        command = [
            "time",
            "cargo",
            "run",
            "--release",
            "--example",
            example_name,
            *sys.argv[1:],
        ]
        print(" ", " ".join(map(quote, command)))
        subprocess.run(command, shell=True, check=True, env=env)

        print()
        print("=============================================")

if __name__ == "__main__":
    try:
        main()
    except CalledProcessError as err:
        if err.stdout:
            print("===== STDOUT =====")
            print(err.stdout)
        if err.stderr:
            print("===== STDERR =====")
            print(err.stderr)

        print("\nProcess returned non-zero exit status", err.returncode)
