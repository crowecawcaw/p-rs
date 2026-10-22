"""Read both live transcripts and losslessly compressed frozen transcripts."""
import gzip

def read_transcript(run):
    path = run / "tool-transcript.jsonl"
    if path.exists():
        return path.read_text()
    with gzip.open(str(path) + ".gz", "rt") as stream:
        return stream.read()
