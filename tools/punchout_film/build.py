"""Build the Punch-Out!! film.

    cd tools && python3 -m punchout_film.build [--chapter N]... [--quick] [--no-music] [--contact]
"""

import sys

from reservoir_film.build import run

from .film import PUNCHOUT

if __name__ == "__main__":
    run(PUNCHOUT, sys.argv[1:])
