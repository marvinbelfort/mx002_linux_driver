#gerado com:
# sudo usbhid-dump -a1:12 -i1 | rg -v : | xxd -r -p | hidrd-convert -o spec > usbhid-descriptor-i1

Usage Page (FFA0h),                 ; FFA0h, vendor-defined
Usage (01h),
Collection (Application),
    Report ID (6),
    Report Count (8),
    Report Size (63),
    Logical Minimum (0),
    Logical Maximum (255),
    Usage (01h),
    Usage Minimum (00h),
    Usage Maximum (FFh),
    Input,
End Collection,
Usage Page (Desktop),               ; Generic desktop controls (01h)
Usage (Mouse),                      ; Mouse (02h, application collection)
Collection (Application),
    Report ID (3),
    Usage (Pointer),                ; Pointer (01h, physical collection)
    Collection (Physical),
        Usage Page (Button),        ; Button (09h)
        Usage Minimum (01h),
        Usage Maximum (08h),
        Logical Minimum (0),
        Logical Maximum (1),
        Report Count (8),
        Report Size (1),
        Input (Variable),
        Usage Page (Desktop),       ; Generic desktop controls (01h)
        Logical Minimum (-127),
        Logical Maximum (127),
        Report Size (8),
        Report Count (3),
        Usage (X),                  ; X (30h, dynamic value)
        Usage (Y),                  ; Y (31h, dynamic value)
        Usage (Wheel),              ; Wheel (38h, dynamic value)
        Input (Variable, Relative),
    End Collection,
End Collection
