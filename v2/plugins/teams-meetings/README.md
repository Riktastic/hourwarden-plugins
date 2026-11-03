# Teams meetings

**Works on:** Windows only: it reads microphone use from the Windows registry and the Teams desktop app, which does not exist on Linux.

Notices Teams calls and meetings, also when Teams is in the background, and gives the meeting time to the customer named in the meeting title.

## What it does

Notices Teams calls and meetings, also when Teams is in the background or you only listen. The meeting title decides the customer, and a meeting for a known customer takes the time whatever else is in front.

It sees that a call is running from which program uses the microphone (Windows privacy settings), and reads the meeting title from the Teams windows.

## Setting it up

Use the customer's name or alias in meeting titles, or let the customer's own invitations do that. Calls without a recognisable title are reported as *Teams call* (*Count calls without a title*); you can then pick the customer when asked.

## What it reads

Which program uses the microphone (read only) and the titles of Teams windows. It does not listen to anything.

## Settings

| Setting | Default | What it does |
|---|---|---|
| Check every | `5` | How often the microphone is checked, in seconds. |
| Count calls without a title | `on` | Also report a meeting when the call window has no recognisable title (the meeting is then named 'Teams call'). |
