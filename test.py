print("Rendering test image...")
write = app1.createWriter("target/filtered_test_####.png")

source = app1.createNode("net.sf.openfx.CheckerBoardPlugin")
mask = app1.createNode("net.sf.openfx.Radial")

under_test = app1.createNode("net.itadinanta.ofx-rs.simple_plugin_1")
under_test.connectInput(0, source)
under_test.connectInput(1, mask)

under_test.getParam("scaleComponents").setValue(True)
under_test.getParam("scale").setValue(1.0)
under_test.getParam("scaleR").setValue(1.5)

write.connectInput(0, under_test)

app1.render(write, 1, 1)

quit()

