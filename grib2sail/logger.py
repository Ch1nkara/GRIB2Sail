import logging
from rich.logging import RichHandler
import sys

logging.basicConfig(
  level=logging.INFO,
  format="%(message)s",
  datefmt="%Y-%m-%d %H:%M:%S",
  handlers=[RichHandler(rich_tracebacks=False)],
)

def error_exit(self, msg, *args, exit_code=1, to_clean=[]):
  self.error(msg, *args)
  for file in to_clean:
    file.unlink(missing_ok=True)
  sys.exit(exit_code)
logging.Logger.error_exit = error_exit

logger = logging.getLogger(__name__)

