# gerado com:
# sudo usbhid-dump -a1:12 -i2 | rg -v : | xxd -r -p | hidrd-convert -o spec > usbhid-descriptor-i2

Usage Page (FFA0h),                             ; FFA0h, vendor-defined
Usage (01h),
Collection (Application),
    Report ID (1),
    Usage (02h),
    Logical Minimum (0),
    Logical Maximum (-1),
    Report Size (8),
    Report Count (7),
    Input (Variable),
    Logical Minimum (0),
    Logical Maximum (1),
    Report Count (64),
    Report Size (1),
    Usage Page (LED),                           ; LEDs (08h)
    Usage Minimum (01h),
    Usage Maximum (40h),
    Output (Variable),
    Usage (00h),
    Logical Minimum (0),
    Logical Maximum (255),
    Report Size (8),
    Report Count (7),
    Report ID (8),
    Feature (Variable),
End Collection,
Usage Page (Desktop),                           ; Generic desktop controls (01h)
Usage (Keyboard),                               ; Keyboard (06h, application collection)
Collection (Application),
    Report ID (2),
    Usage Page (Keyboard),                      ; Keyboard/keypad (07h)
    Usage Minimum (KB Leftcontrol),             ; Keyboard left control (E0h, dynamic value)
    Usage Maximum (KB Right GUI),               ; Keyboard right GUI (E7h, dynamic value)
    Logical Minimum (0),
    Logical Maximum (1),
    Report Size (1),
    Report Count (8),
    Input (Variable),
    Report Count (1),
    Report Size (8),
    Input (Constant),
    Report Count (5),
    Report Size (1),
    Usage Page (LED),                           ; LEDs (08h)
    Usage Minimum (01h),
    Usage Maximum (05h),
    Output (Variable),
    Report Count (1),
    Report Size (3),
    Output (Constant),
    Report Count (5),
    Report Size (8),
    Logical Minimum (0),
    Logical Maximum (101),
    Usage Page (Keyboard),                      ; Keyboard/keypad (07h)
    Usage Minimum (None),                       ; No event (00h, selector)
    Usage Maximum (KB Application),             ; Keyboard Application (65h, selector)
    Input,
End Collection,
Usage Page (Digitizer),                         ; Digitizer (0Dh)
Usage (Digitizer),                              ; Digitizer (01h, application collection)
Collection (Application),
    Report ID (5),
    Usage (Stylus),                             ; Stylus (20h, application collection, logical collection)
    Collection (Physical),
        Usage (Tip Switch),                     ; Tip switch (42h, momentary control)
        Usage (Barrel Switch),                  ; Barrel switch (44h, momentary control)
        Usage (Eraser),                         ; Eraser (45h, momentary control)
        Usage (Invert),                         ; Invert (3Ch, momentary control)
        Logical Minimum (0),
        Logical Maximum (1),
        Report Size (1),
        Report Count (4),
        Input (Variable),
        Report Size (1),
        Report Count (2),
        Input (Constant),
        Usage (In Range),                       ; In range (32h, momentary control)
        Report Size (1),
        Report Count (1),
        Input (Variable),
        Input (Constant),
        Usage Page (Desktop),                   ; Generic desktop controls (01h)
        Usage (X),                              ; X (30h, dynamic value)
        Report Size (16),
        Report Count (1),
        Push,
        Unit Exponent (13),
        Unit (Inch),
        Physical Minimum (0),
        Physical Maximum (8000),
        Logical Maximum (4096),
        Input (Variable),
        Usage (Y),                              ; Y (31h, dynamic value)
        Physical Maximum (5340),
        Logical Maximum (4096),
        Input (Variable),
        Pop,
        Usage Page (Digitizer),                 ; Digitizer (0Dh)
        Usage (Tip Pressure),                   ; Tip pressure (30h, dynamic value)
        Logical Maximum (2047),
        Physical Maximum (2047),
        Unit Exponent (0),
        Unit (Centimeter * Gram * Seconds^-2),
        Report Size (16),
        Input (Variable),
    End Collection,
End Collection,
Usage Page (Consumer),                          ; Consumer (0Ch)
Usage (Consumer Control),                       ; Consumer control (01h, application collection)
Collection (Application),
    Report ID (4),
    Usage Minimum (00h),
    Usage Maximum (AC Format),                  ; AC format (023Ch, selector)
    Logical Minimum (0),
    Logical Maximum (572),
    Report Count (1),
    Report Size (16),
    Input,
End Collection,
Usage Page (Digitizer),                         ; Digitizer (0Dh)
Usage (Pen),                                    ; Pen (02h, application collection)
Collection (Application),
    Report ID (7),
    Usage (Stylus),                             ; Stylus (20h, application collection, logical collection)
    Collection (Physical),
        Usage (Tip Switch),                     ; Tip switch (42h, momentary control)
        Usage (Barrel Switch),                  ; Barrel switch (44h, momentary control)
        Usage (Eraser),                         ; Eraser (45h, momentary control)
        Usage (Invert),                         ; Invert (3Ch, momentary control)
        Usage (Secondary Tip Switch),           ; Secondary tip switch (43h, momentary control)
        Usage (Barrel Switch),                  ; Barrel switch (44h, momentary control)
        Logical Minimum (0),
        Logical Maximum (1),
        Report Size (1),
        Report Count (6),
        Input (Variable),
        Usage (In Range),                       ; In range (32h, momentary control)
        Report Size (1),
        Report Count (1),
        Input (Variable),
        Input (Constant, Variable),
        Usage Page (Desktop),                   ; Generic desktop controls (01h)
        Usage (X),                              ; X (30h, dynamic value)
        Usage (Y),                              ; Y (31h, dynamic value)
        Unit Exponent (13),
        Unit (Inch^3),
        Logical Maximum (4096),
        Physical Minimum (0),
        Physical Maximum (8000),
        Report Size (16),
        Report Count (2),
        Input (Variable),
        Usage Page (Digitizer),                 ; Digitizer (0Dh)
        Usage (Tip Pressure),                   ; Tip pressure (30h, dynamic value)
        Physical Maximum (8191),
        Logical Maximum (2047),
        Report Size (16),
        Report Count (1),
        Input (Variable),
    End Collection,
End Collection
