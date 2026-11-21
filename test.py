import gc

from PySide import QtCore

print("Rendering test image...")
write = app1.createWriter("target/filtered_test_####.png")

source = app1.createNode("net.sf.openfx.CheckerBoardPlugin")
mask = app1.createNode("net.sf.openfx.Radial")

under_test = app1.createNode("net.itadinanta.ofx-rs.basic")
under_test.connectInput(0, source)
under_test.connectInput(1, mask)

under_test.getParam("scaleComponents").setValue(True)
under_test.getParam("scale").setValue(1.0)
under_test.getParam("scaleR").setValue(1.5)

write.connectInput(0, under_test)

app1.render(write, 1, 1)

# Remove the node under test and flush its deferred deletion so the
# plugin receives kOfxActionDestroyInstance. Headless Natron has a
# QCoreApplication but no running event loop, so destroy() alone never
# completes: gc.collect() drops the wrapper's reference and
# processEvents() runs Qt's deferred-delete queue.
under_test.destroy()
gc.collect()
QtCore.QCoreApplication.processEvents()

# Close the project afterwards: the already-destroyed node must not
# receive a second kOfxActionDestroyInstance.
app1.closeProject()
gc.collect()
QtCore.QCoreApplication.processEvents()

quit()

