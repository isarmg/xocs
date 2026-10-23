"""Shared receipt-verified staging for formal and installed-candidate packages."""
import hashlib
import json
from pathlib import Path
import os
import re
import shutil

PRODUCT = "blog"
TARGET = "x86_64-unknown-linux-gnu"
SOURCE = Path(__file__).resolve().parents[2]


# Kept in the shipped package verifier so installed checks use exactly the same
# canonical identity and executable binding as source staging.
import sys
sys.path.insert(0, str(SOURCE / "src/tools/packaging"))
from package import canonical_build_identity, read_regular, require_compiled_identity


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


def stage_documentation(destination:Path,revision:str,*,source:Path=SOURCE)->None:
    """Keep installation and operations manuals usable outside the source tree."""
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("documentation must identify its reviewed source revision")
    manuals = {
        "deploy/linux-x86_64/README.md": "README.md",
        **{f"docs/{name}.md": f"docs/{name}.md" for name in
           ("getting-started", "configuration", "usage", "operations", "contracts")},
    }
    for origin, relative in manuals.items():
        value = read_regular(source / origin).decode("utf-8")
        if relative == "README.md":
            value = value.replace("](../../docs/", "](docs/")
        else:
            value = value.replace("](../deploy/linux-x86_64/README.md)", "](../README.md)")
            # Building the project requires the full checkout, rather than the
            # runtime package. Keep that reference bound to the packaged source.
            value = value.replace("](development.md)",
                                  f"](https://github.com/isarmg/xocs/blob/{revision}/docs/development.md)")
        target = destination / relative
        target.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
        with target.open("xb") as stream:
            stream.write(value.encode("utf-8"))
        target.chmod(0o644)


if __name__ == "__main__":
    import sys
    binary, destination, revision, source = sys.argv[1:]
    stage_verified_server(Path(binary), Path(destination) / "bin" / PRODUCT, revision)
    stage_original_notices(Path(destination) / "share", source=Path(source))
