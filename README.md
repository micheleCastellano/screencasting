# Project m1: Multi-platform screen-casting

This desktop application was developed as part of the final project for the System Programming course at Politecnino di Torino.
The project was designed to meet the requirements and guidelines provided by our instructor, which are included below.

## Instructor guidelines
Using the Rust programming language, create a screencasting application capable of continuously
grabbing the content of the screen (or a portion of it) and stream it to a set of peers.

The application should fulfill the following requirements:

1. Platform Support: The utility should be compatible with multiple desktop operating systems,
including Windows, macOS, and Linux.

2. User Interface (UI): The utility should have an intuitive and user-friendly interface that allows
users to easily navigate through the application's features.

3. User Interface (UI): The utility should have an intuitive and user-friendly interface that allows
users to easily navigate through the application's features.

4. Selection Options: When in casting mode, the utility should allow the user to restrict the
grabbed content to a custom area.

5. Operating mode: At startup, the user will choose whether the application should operate as a
caster or as a receiver. In the latter case, the user should be able to specify the address of the
caster it should connect to.

7. Selection Options: When in casting mode, the utility should allow the user to restrict the
grabbed content to a custom area.

8. Hotkey Support: The utility should support customizable keyboard shortcuts for
pausing/resuming the transmission, for blanking the screen and terminating the current session.
As a bonus, the application may also provide the following features:

9. Annotation Tools: When in casting mode, the utility can activate/deactivate a transparent
layer on top of the grabbed area where annotations like shapes, arrows, text, …, can be
superimposed to the original content.

10. Save Options: When in receiving mode, the utility should allow users to record the received
content to a video file.

11. Multi-monitor Support: The utility should be able to recognize and handle
multiple monitors independently, allowing users to cast content from any of the connected
displays.
