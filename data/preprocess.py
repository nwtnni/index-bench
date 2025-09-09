import click
import rarfile
import string
import typing
from tqdm import tqdm
import zipfile


IGNORE = [
    "china/area/readme.gif",
    "china/area/thumbs.db",
    "commercial/adult/adult1.fields",
    "commercial/adult/adult1.msg",
    "commercial/adult/adult1.prh",
    # XLS format
    "personal profile/46 mil compañias argentinas.xls",
    "personal profile/50 mil brasil.xls",
    "personal profile/65 mil argentina.xls",
]


@click.command()
@click.option(
    "--input",
    default="300 million email database.rar",
    help="Path to email database",
)
@click.option(
    "--output",
    default="email.txt",
    help="Path to write output",
)
@click.option(
    "--count",
    default=16000000,
    help="Number of emails to extract",
)
def main(input, output, count):
    progress = tqdm(range(count))

    with open(output, "x") as file:
        for _, email in zip(range(count), iter_email(progress, input)):
            file.write(email)
            file.write("\n")
            progress.update(1)


def iter_email(progress, root):
    seen = set()

    for email in iter_file(progress, root):
        canonical = canonicalize(email)

        if canonical is None:
            progress.write(f"Failed to canonicalize email: {email}")
            continue

        if canonical in seen:
            continue

        seen.add(canonical)
        yield canonical


def iter_file(progress, root):
    with rarfile.RarFile(root) as archive:
        for index, (archive, info) in enumerate(iter_info(progress, archive)):
            with archive.open(info) as file:
                for email in file:
                    yield from email.split()


def iter_info(progress, archive):
    for info in archive.infolist():
        if (
            # RarFile
            hasattr(info, "isdir")
            and info.isdir()
            # ZipFile
            or hasattr(info, "is_dir")
            and info.is_dir()
        ):
            continue

        name = info.filename.lower()

        if name in IGNORE:
            progress.write(f"Ignoring file: {name}")
            continue
        elif name.endswith(".zip"):
            with zipfile.ZipFile(archive.open(info)) as file:
                progress.write(f"Recursing into {info.filename}")
                yield from iter_info(progress, file)
        elif name.endswith(".rar"):
            with rarfile.RarFile(archive.open(info)) as file:
                progress.write(f"Recursing into {info.filename}")
                yield from iter_info(progress, file)
        else:
            yield (archive, info)


def canonicalize(email: bytes) -> typing.Optional[str]:
    try:
        email = email.decode("utf-8")
    except UnicodeError:
        return None

    if "@" not in email:
        return None

    username, domain = email.lower().split("@", 1)
    return f"{'.'.join(reversed(domain.split('.')))}@{username}"


if __name__ == "__main__":
    main()
