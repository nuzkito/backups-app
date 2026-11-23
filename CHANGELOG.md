# Changelog

Backups App is a portable folder backup tool for Windows. It saves your folders to a single ZIP
file. This page tells you what is new in each version.

## 0.1.0 - 2026-11-23

This is the first version of Backups App. You choose your folders once, and from then on one click
saves all of them to a new ZIP file.

### Back up all your folders with one click

Add the folders you want to keep safe, choose a destination (for example, an external drive) and
click **Back up now**. Backups App saves everything to a single file named with the date and time,
such as `backup-2026-11-23-18-30-00.zip`. Each backup is a new file, so your earlier backups stay
as they were.

Backups App remembers your folders. The next time, open the application and click **Back up now**.

### Skip the files you do not need

Add names such as `node_modules`, `*.tmp` or `Thumbs.db` to **Exclusions**, and Backups App skips
them. Your backups are smaller and finish sooner.

### No broken backups, and you see which files Backups App skipped

Backups App never replaces a file in the destination folder. If you cancel a backup, or if it fails,
Backups App removes the incomplete file, so you never find a broken ZIP next to your good ones.

If Backups App cannot open a file (for example, because a different program uses it), it continues
with the other files. At the end, it shows you which files it skipped, so you can close that
program and back up again.

### No time lost on files that are already compressed

Photos, videos, music, PDF files and Word, Excel and PowerPoint files are already compressed.
Backups App copies them as they are, because a second compression does not make them smaller.

### Run Backups App without installing it

Download `backups-0.1.0-portable.zip`, extract `backups.exe` and run it. To use Backups App on a
different computer or from a USB drive, copy its folder.
