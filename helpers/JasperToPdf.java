import java.io.*;
import java.util.zip.GZIPInputStream;
import net.sf.jasperreports.engine.JasperPrint;
import net.sf.jasperreports.engine.JasperExportManager;
import net.sf.jasperreports.engine.xml.JRPrintXmlLoader;

public class JasperToPdf {
    public static void main(String[] args) {
        if (args.length < 2) {
            System.err.println("Usage: JasperToPdf <input.xml|input.gz> <output.pdf>");
            System.exit(1);
        }
        try {
            File inputFile = new File(args[0]);
            File outputFile = new File(args[1]);

            InputStream is = new FileInputStream(inputFile);
            // Check if gzipped
            byte[] magic = new byte[2];
            PushbackInputStream pbis = new PushbackInputStream(is, 2);
            int read = pbis.read(magic);
            if (read == 2) {
                pbis.unread(magic);
                if ((magic[0] == (byte) 0x1f) && (magic[1] == (byte) 0x8b)) {
                    is = new GZIPInputStream(pbis);
                } else {
                    is = pbis;
                }
            } else {
                is = pbis;
            }

            JasperPrint jp = JRPrintXmlLoader.load(is);
            is.close();
            JasperExportManager.exportReportToPdfFile(jp, outputFile.getAbsolutePath());
            System.out.println("SUCCESS: " + outputFile.getAbsolutePath());
        } catch (Exception e) {
            e.printStackTrace();
            System.exit(2);
        }
    }
}
