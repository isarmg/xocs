"""Shared receipt-verified staging for formal and installed-candidate packages."""
import hashlib
import json
from pathlib import Path
import os
import re
import shutil

PRODUCT = "blog"
TARGET = "x86_64-unknown-linux-gnu"
SOURCE = Path(__file__).resolve().parents[1]


# Kept in the shipped package verifier so installed checks use exactly the same
# canonical identity and executable binding as source staging.
import sys
sys.path.insert(0, str(SOURCE / "tools/packaging"))
from package import canonical_build_identity, require_compiled_identity


def stage_verified_server(binary:Path,destination:Path,revision:str)->None:
    # The receipt is written by the trusted builder after Cargo returns this ELF.
    receipt=json.loads(Path(str(binary)+".blog-assets.json").read_bytes())
    if (not isinstance(receipt,dict) or receipt.get("format")!="blog-assets-receipt-v1"
            or not re.fullmatch(r"[0-9a-f]{64}",receipt.get("binary_sha256", ""))
            or not isinstance(receipt.get("identity"),dict)
            or receipt["identity"].get("source_revision")!=revision
            or receipt["identity"].get("product")!=PRODUCT
            or receipt["identity"].get("target")!=TARGET):
        raise ValueError("server build receipt differs from the reviewed product source")
    created=False
    try:
        with binary.open("rb") as source,destination.open("xb") as output:
            created=True
            shutil.copyfileobj(source,output)
            output.flush();os.fsync(output.fileno())
        with destination.open("rb") as source:
            digest=hashlib.file_digest(source,"sha256").hexdigest()
        if digest!=receipt["binary_sha256"]:
            raise ValueError("staged server differs from its exact build receipt")
        destination.chmod(0o755)
    except BaseException:
        if created:destination.unlink(missing_ok=True)
        raise

def stage_original_notices(destination:Path,*,source:Path=SOURCE)->None:
    shutil.copyfile(source/"LICENSE",destination/"LICENSE")
    (destination/"LICENSE").chmod(0o644)


if __name__ == "__main__":
    import sys
    binary, destination, revision, source = sys.argv[1:]
    stage_verified_server(Path(binary), Path(destination) / "bin" / PRODUCT, revision)
    stage_original_notices(Path(destination) / "share", source=Path(source))
