# git (Rust)

A simplified implementation of Git core commands written in Rust, inspired by the [Write Yourself a Git](https://wyag.thb.lt/) guide.

This project reimplements Git's internals from the ground up — object storage, the index file, reference management, and the staging/commit workflow — producing a CLI that is compatible with real Git repositories.

## Features

- **Repository management**: `init`
- **Object storage** (blob, tree, commit, tag): `hash-object`, `cat-file`
- **History traversal**: `log` 
- **Tree inspection & checkout**: `ls-tree`, `checkout`
- **References & tags**: `show-ref`, `tag`, `rev-parse`
- **Staging area**: `ls-files`, `status`, `add`, `rm`
- **Committing**: `commit`
- **`.gitignore` support**: `check-ignore`

## Project Structure

```
src/
├── main.rs                  # CLI entry point (clap-based argument parsing)
├── repository.rs            # Core repository: init, find, read/write objects, refs, index operations
├── index.rs                 # Git index (staging area) parser and serializer (DIRC format, v2)
├── config.rs                # Git config file (.git/config) parser
├── git_ignore.rs            # .gitignore rule parsing and matching
├── commands/                # One module per subcommand (add, checkout, commit, etc.)
│   ├── add.rs
│   ├── cat_file.rs
│   ├── check_ignore.rs
│   ├── checkout.rs
│   ├── commit.rs
│   ├── hash_object.rs
│   ├── init.rs
│   ├── log.rs
│   ├── ls_files.rs
│   ├── ls_tree.rs
│   ├── rev_parse.rs
│   ├── rm.rs
│   ├── show_ref.rs
│   ├── status.rs
│   └── tag.rs
├── objects/                 # Git object types
│   ├── blob.rs              # Blob (file content)
│   ├── commit.rs            # Commit (tree + parent + author/message)
│   ├── tree.rs              # Tree (directory listing)
│   ├── tag.rs               # Tag (annotated tag)
│   └── shared/              # Shared object traits and types
│       ├── object.rs        # Object trait
│       ├── object_type.rs   # ObjectType enum
│       └── compressed_object.rs
└── utils/                   # Utility modules
    ├── sha1.rs              # SHA-1 hashing
    ├── zlib.rs              # zlib compression/decompression
    └── kvlm.rs              # Key-Value List with Message (commit/tag format parser)
```

## Architecture

### Object Storage

Git objects are stored in `.git/objects/` using the SHA-1 hash as the filename (split into a 2-character directory prefix + 38-character filename). Each object is prefixed with a type header (`blob`, `tree`, `commit`, `tag`), a size, a null byte, and the raw content — all compressed with zlib.

### Object Types

| Type | Contents |
|------|----------|
| **Blob** | Raw file data |
| **Tree** | Sorted list of entries (mode, SHA, filename), representing a directory |
| **Commit** | KVLM-format: tree SHA, parent SHA(s), author, committer, message |
| **Tag** | KVLM-format: object SHA, type, tag name, tagger, message |

### Index (Staging Area)

The index file (`.git/index`) uses the DIRC format (v2). It stores file metadata (ctime, mtime, dev, inode, mode, uid, gid, size), the SHA-1 hash of the staged content, and the filename.

### References

References are stored as files under `.git/refs/` (heads, tags) or as symbolic refs (`ref: refs/heads/master`). The `HEAD` file can be either a symbolic ref or a direct SHA-1 (detached HEAD).

## Usage

```
git init [path]
git hash-object [-w] [-t TYPE] FILE
git cat-file TYPE OBJECT
git log [commit]
git ls-tree [-r] [tree]
git checkout COMMIT PATH
git show-ref
git tag [-a] [name] [object]
git rev-parse [-t TYPE] NAME
git ls-files [--verbose]
git check-ignore PATH...
git status
git rm PATH...
git add PATH...
git commit -m MESSAGE
```

## Building

```bash
cargo build --release
```

## Dependencies

- `sha1` — SHA-1 hashing
- `flate2` — zlib compression
- `clap` — CLI argument parsing
- `anyhow` — Error handling
- `chrono` — Timestamps
- `regex` — Pattern matching
- `hex` — Hex encoding/decoding
- `indexmap` — Ordered map for KVLM serialization

## License

MIT
